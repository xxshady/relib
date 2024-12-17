use std::{
  collections::HashMap,
  sync::{LazyLock, Mutex, MutexGuard},
};

use relib_internal_shared::{
  Allocation, AllocatorOp, AllocatorPtr, ModuleId, SliceAllocatorOp, StableLayout,
};

use crate::{exports_types::ModuleExportsForHost, helpers::unrecoverable, Module};

type Allocs = HashMap<ModuleId, HashMap<AllocatorPtr, Allocation>>;

static ALLOCS: LazyLock<Mutex<Allocs>> = LazyLock::new(|| Mutex::new(HashMap::new()));

fn lock_allocs() -> MutexGuard<'static, Allocs> {
  let Ok(allocs) = ALLOCS.lock() else {
    unrecoverable("failed to lock ALLOCS");
  };

  allocs
}

pub fn add_module(module_id: ModuleId) {
  let mut allocs = lock_allocs();
  allocs.insert(module_id, Default::default());
}

pub fn remove_module<E: ModuleExportsForHost>(module: &Module<E>) {
  unsafe {
    module.internal_exports.take_cached_allocs_before_exit();
  }

  let mut allocs = lock_allocs();
  let Some(allocs) = allocs.remove(&module.id) else {
    panic!("Failed to take allocs of module with id: {}", module.id);
  };

  let allocs: Box<[Allocation]> = allocs.into_values().collect();
  let allocs: &[Allocation] = &allocs;

  unsafe {
    module.internal_exports.exit(allocs.into());
  }
}

pub extern "C" fn on_cached_allocs(module_id: ModuleId, ops: SliceAllocatorOp) {
  let ops = unsafe { ops.into_slice() };

  let mut allocs = lock_allocs();
  let allocs = allocs.get_mut(&module_id).unwrap_or_else(|| unreachable!());

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

pub extern "C" fn on_alloc(module_id: ModuleId, ptr: *mut u8, layout: StableLayout) {
  let mut allocs = lock_allocs();
  let allocs = allocs
    .get_mut(&module_id)
    .unwrap_or_else(|| unrecoverable("on_alloc unreachable"));

  let ptr = AllocatorPtr(ptr);
  allocs.insert(ptr, Allocation(ptr, layout));
}
