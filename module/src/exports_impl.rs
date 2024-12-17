use std::sync::atomic::Ordering;

use relib_internal_shared::{exports::___Internal___Exports___ as Exports, ModuleId};
use crate::{
  alloc_tracker, gen_exports::ModuleExportsImpl, panic_hook, ALLOCATOR_LOCK, HOST_OWNER_THREAD,
  MODULE_ID,
};

impl Exports for ModuleExportsImpl {
  fn init(host_owner_thread: usize, module: ModuleId) {
    unsafe {
      HOST_OWNER_THREAD = host_owner_thread;
      MODULE_ID = module;

      alloc_tracker::init();
    }

    panic_hook::init();
  }

  fn exit(allocs: relib_internal_shared::SliceAllocation) {
    unsafe {
      let allocs = allocs.into_slice();
      alloc_tracker::dealloc(allocs);
    }
  }

  fn take_cached_allocs_before_exit() {
    alloc_tracker::send_cached_allocs(None);
  }

  fn lock_module_allocator() {
    ALLOCATOR_LOCK.store(true, Ordering::SeqCst);
  }

  fn run_thread_local_dtors() {
    #[cfg(target_os = "linux")]
    {
      use crate::thread_locals;
      unsafe {
        thread_locals::dtors::run();
      }
    }
  }

  fn spawned_threads_count() -> u64 {
    #[cfg(target_os = "linux")]
    {
      crate::thread_spawn_hook::spawned_threads_count()
    }
    #[cfg(target_os = "windows")]
    {
      Default::default()
    }
  }
}
