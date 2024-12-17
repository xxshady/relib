use std::{
  alloc::{GlobalAlloc, Layout},
  collections::HashMap,
  sync::{
    atomic::{AtomicBool, Ordering},
    LazyLock, Mutex, MutexGuard,
  },
};

use relib_internal_shared::{Allocation, AllocatorOp, AllocatorPtr, StableLayout};

use crate::{
  gen_imports,
  helpers::{assert_allocator_is_still_accessible, unrecoverable},
  MODULE_ID,
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
    #[cfg(target_os = "windows")]
    {
      use crate::helpers::disable_allocator_for_thread_local_destructors;
      if disable_allocator_for_thread_local_destructors() {
        unrecoverable(
          "module cannot allocate after its memory has been freed\n\
          note: check if thread-locals registered in main thread of the module have allocations \
          inside Drop implementation, since currently it's not supported on windows",
        );
      }
    }

    assert_allocator_is_still_accessible();

    let ptr = self.allocator.alloc(layout);

    let c_layout = StableLayout {
      size: layout.size(),
      align: layout.align(),
    };

    if ALLOC_INIT.load(Ordering::SeqCst) {
      gen_imports::on_alloc(MODULE_ID, ptr, c_layout);
    } else {
      save_alloc_in_cache(ptr, c_layout);
    }

    ptr
  }

  unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
    #[cfg(target_os = "windows")]
    {
      use crate::helpers::disable_allocator_for_thread_local_destructors;
      if disable_allocator_for_thread_local_destructors() {
        return;
      }
    }

    assert_allocator_is_still_accessible();

    self.allocator.dealloc(ptr, layout);

    if !UNLOAD_DEALLOCATION.load(Ordering::SeqCst) {
      let c_layout = StableLayout {
        size: layout.size(),
        align: layout.align(),
      };

      save_dealloc_in_cache(ptr, c_layout);
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
      std::alloc::dealloc(
        *ptr,
        Layout::from_size_align(layout.size, layout.align).unwrap_or_else(|_| unreachable!()),
      );
    }
  }

  UNLOAD_DEALLOCATION.swap(false, Ordering::SeqCst);
}
