use {
  super::{
    MODULE_ID, gen_imports,
    helpers::{assert_allocator_is_still_accessible, unrecoverable},
  },
  relib_internal_shared::{Allocation, AllocatorOp, AllocatorPtr, StableLayout},
  std::{
    alloc::{GlobalAlloc, Layout},
    collections::HashMap,
    sync::{
      LazyLock, Mutex, MutexGuard,
      atomic::{AtomicBool, Ordering},
    },
  },
};

static UNLOAD_DEALLOCATION: AtomicBool = AtomicBool::new(false);

#[derive(Default, Debug)]
pub struct AllocTracker<A: GlobalAlloc> {
  allocator: A,
}

impl<A: GlobalAlloc> AllocTracker<A> {
  pub const fn new(allocator: A) -> Self {
    AllocTracker { allocator }
  }
}

unsafe impl<A: GlobalAlloc> GlobalAlloc for AllocTracker<A> {
  unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
    assert_allocator_is_still_accessible();

    let ptr = unsafe { self.allocator.alloc(layout) };

    if ALLOC_INIT.load(Ordering::SeqCst) {
      // TODO: SAFETY
      unsafe {
        gen_imports::on_alloc(MODULE_ID, ptr, layout.into());
      }
    } else {
      save_alloc_in_cache(ptr, layout.into());
    }

    ptr
  }

  unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
    assert_allocator_is_still_accessible();

    #[cfg(feature = "dealloc_validation")]
    if !UNLOAD_DEALLOCATION.load(Ordering::SeqCst) && !is_ptr_valid(ptr) {
      unrecoverable("invalid pointer was passed to dealloc of global allocator");
    }

    // TODO: SAFETY
    unsafe {
      self.allocator.dealloc(ptr, layout);
    }

    if !UNLOAD_DEALLOCATION.load(Ordering::SeqCst) {
      save_dealloc_in_cache(ptr, layout.into());
    }
  }
}

const CACHE_SIZE: usize = 20_000;

type AllocsCache = HashMap<AllocatorPtr, AllocatorOp>;

static ALLOCS_CACHE: LazyLock<Mutex<AllocsCache>> = LazyLock::new(|| Mutex::new(HashMap::new()));
static ALLOC_INIT: AtomicBool = AtomicBool::new(false);

static TRANSPORT_BUFFER: Mutex<Vec<AllocatorOp>> = Mutex::new(Vec::new());

fn lock_allocs_cache() -> MutexGuard<'static, AllocsCache> {
  ALLOCS_CACHE.lock().unwrap_or_else(|_| {
    unrecoverable("failed to lock ALLOCS_CACHE");
  })
}

fn lock_transport_buffer() -> MutexGuard<'static, Vec<AllocatorOp>> {
  TRANSPORT_BUFFER.lock().unwrap_or_else(|_| {
    unrecoverable("failed to lock TRANSPORT_BUFFER");
  })
}

fn push_to_allocs_cache(op: AllocatorOp, cache: Option<&mut AllocsCache>) {
  let cache = if let Some(cache) = cache {
    cache
  } else {
    &mut lock_allocs_cache()
  };

  let ptr = match op {
    AllocatorOp::Alloc(Allocation(ptr, ..)) => ptr,
    AllocatorOp::Dealloc(Allocation(ptr, ..)) => ptr,
  };

  cache.insert(ptr, op);

  if cache.len() == CACHE_SIZE {
    send_cached_allocs(Some(cache));
  }
}

fn save_alloc_in_cache(ptr: *mut u8, layout: StableLayout) {
  push_to_allocs_cache(
    AllocatorOp::Alloc(Allocation(AllocatorPtr(ptr), layout)),
    None,
  );
}

fn save_dealloc_in_cache(ptr: *mut u8, layout: StableLayout) {
  let cache = &mut lock_allocs_cache();

  let ptr = AllocatorPtr(ptr);
  push_to_allocs_cache(AllocatorOp::Dealloc(Allocation(ptr, layout)), Some(cache));
}

pub unsafe fn init() {
  ALLOC_INIT.swap(true, Ordering::SeqCst);

  // !!! keep in mind that at least one of these allocations is needed for AllocTracker check in host !!!

  let mut cache = lock_allocs_cache();
  cache.reserve(CACHE_SIZE);

  let mut transport = lock_transport_buffer();
  transport.reserve(CACHE_SIZE);

  ALLOC_INIT.swap(false, Ordering::SeqCst);
}

pub fn send_cached_allocs(cache: Option<&mut AllocsCache>) {
  let cache = if let Some(cache) = cache {
    cache
  } else {
    &mut lock_allocs_cache()
  };

  let mut transport = lock_transport_buffer();

  transport.extend(cache.drain().map(|(_, allocation)| allocation));

  unsafe {
    let slice: &[AllocatorOp] = &transport;
    gen_imports::on_cached_allocs(MODULE_ID, slice.into());
  }

  transport.clear();
}

pub unsafe fn dealloc(allocs: &[Allocation]) {
  UNLOAD_DEALLOCATION.swap(true, Ordering::SeqCst);

  for Allocation(AllocatorPtr(ptr), layout) in allocs {
    unsafe {
      std::alloc::dealloc(*ptr, layout.into());
    }
  }

  UNLOAD_DEALLOCATION.swap(false, Ordering::SeqCst);
}

#[cfg(feature = "dealloc_validation")]
/// is this pointer allocated by this allocator and is still alive?
fn is_ptr_valid(ptr: *mut u8) -> bool {
  let cache_contains_ptr = {
    let cache = lock_allocs_cache();
    cache.contains_key(&AllocatorPtr(ptr))
  };

  cache_contains_ptr || unsafe { gen_imports::is_ptr_allocated(MODULE_ID, ptr) }
}

#[cfg(not(feature = "dealloc_validation"))]
#[expect(unreachable_code)]
fn _suppress_warn() {
  unsafe { gen_imports::is_ptr_allocated(unreachable!(), unreachable!()) }
}
