use {
  super::{InternalModuleExports, helpers::unrecoverable},
  relib_internal_shared::{Allocation, AllocatorOp, AllocatorPtr, SliceAllocatorOp, StableLayout},
  relib_shared::ModuleId,
  std::{
    alloc::Layout,
    collections::HashMap,
    sync::{LazyLock, Mutex, MutexGuard},
  },
};

type RemovePtrFromCache = fn(ptr: *mut u8);

struct ModuleAllocs {
  map: HashMap<AllocatorPtr, Allocation>,
  remove_ptr_from_cache: RemovePtrFromCache,
}

type Allocs = HashMap<ModuleId, ModuleAllocs>;

static ALLOCS: LazyLock<Mutex<Allocs>> = LazyLock::new(Default::default);

fn lock_allocs() -> MutexGuard<'static, Allocs> {
  let Ok(allocs) = ALLOCS.lock() else {
    unrecoverable("failed to lock ALLOCS");
  };

  allocs
}

pub fn add_module(module_id: ModuleId, remove_ptr_from_cache: RemovePtrFromCache) {
  let mut allocs = lock_allocs();
  allocs.insert(
    module_id,
    ModuleAllocs {
      map: Default::default(),
      remove_ptr_from_cache,
    },
  );
}

pub fn remove_module(
  module_id: ModuleId,
  internal_exports: &InternalModuleExports,
  library_path_str: &str,
  alloc_tracker_enabled: bool,
) {
  unsafe {
    internal_exports.take_cached_allocs_before_exit();
  }

  let mut allocs = lock_allocs();
  let Some(allocs) = allocs.remove(&module_id) else {
    panic!("Failed to take allocs of module with id: {module_id}");
  };

  // this check relies on two allocations in alloc tracker of the module,
  // which needed to cache allocation ops
  if alloc_tracker_enabled && allocs.map.is_empty() {
    eprintln!(
      "[relib] warning: seems like this module doesn't have a registered global alloc tracker\n\
      module path: {}\n\
      note: if \"global_alloc_tracker\" feature is disabled, \
      make sure that you registered relib_module::AllocTracker<A> using #[global_allocator]\n\
      note: if you're sure of what you're doing, you can disable this warning by using \
      `relib_host::load_module_with_options`",
      library_path_str
    );
  }

  let allocs: Box<[Allocation]> = allocs.map.into_values().collect();
  let allocs: &[Allocation] = &allocs;

  // TEST
  println!(
    "calling dealloc for:\n\
    {:?}",
    allocs.iter().map(|v| v).collect::<Vec<_>>()
  );

  unsafe {
    internal_exports.exit(allocs.into());
  }
}

pub fn on_cached_allocs(module_id: ModuleId, ops: SliceAllocatorOp) {
  let ops = unsafe { ops.into_slice() };

  let mut allocs = lock_allocs();
  let allocs = allocs
    .get_mut(&module_id)
    .unwrap_or_else(|| unrecoverable("on_cached_allocs unreachable"));
  let allocs = &mut allocs.map;

  for op in ops {
    match op {
      AllocatorOp::Alloc(allocation) => {
        let Allocation(ptr, ..) = allocation;
        allocs.insert(*ptr, allocation.clone());
      }
      AllocatorOp::Dealloc(Allocation(ptr, ..)) => {
        // doesnt matter if allocs didnt have it
        let _ = allocs.remove(ptr);
      }
    }
  }
}

pub fn on_alloc(module_id: ModuleId, ptr: *mut u8, layout: StableLayout) {
  let mut allocs = lock_allocs();
  let allocs = allocs
    .get_mut(&module_id)
    .unwrap_or_else(|| unrecoverable("on_alloc unreachable"));
  let allocs = &mut allocs.map;

  let ptr = AllocatorPtr(ptr);
  allocs.insert(ptr, Allocation(ptr, layout));
}

pub fn is_ptr_allocated(module_id: ModuleId, ptr: *mut u8) -> bool {
  let allocs = lock_allocs();
  let allocs = allocs
    .get(&module_id)
    .unwrap_or_else(|| unrecoverable("is_ptr_allocated unreachable"));
  let allocs = &allocs.map;

  allocs.contains_key(&AllocatorPtr(ptr))
}

pub fn transfer_alloc_to_host(module_id: ModuleId, ptr: *mut u8) -> bool {
  let mut allocs = lock_allocs();
  let allocs = allocs
    .get_mut(&module_id)
    .unwrap_or_else(|| unrecoverable("transfer_alloc_to_host unreachable"));
  let allocs = &mut allocs.map;

  let ptr = AllocatorPtr(ptr);
  allocs.remove(&ptr).is_some()
}

pub fn transfer_alloc_to_module(ptr: *mut u8, layout: Layout, module_id: relib_shared::ModuleId) {
  {
    let mut allocs = lock_allocs();
    let allocs = allocs
      .get_mut(&module_id)
      .unwrap_or_else(|| unrecoverable("transfer_alloc_to_host unreachable"));

    (allocs.remove_ptr_from_cache)(ptr);

    let ptr = AllocatorPtr(ptr);

    // TODO: get rid of StableLayout if we can just use std types as they are
    // "guaranteed" to be ABI-stable between two crates compiled with the same rustc version + flags
    allocs.map.insert(ptr, Allocation(ptr, layout.into()));
  }
}
