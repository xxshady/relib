use {
  super::{
    ALLOCATOR_LOCK, HOST_OWNER_THREAD, MODULE_ID, alloc_tracker, gen_exports::ModuleExportsImpl,
  },
  crate::host_alloc_proxy::HOST_ALLOC_PROXY,
  relib_internal_shared::{Alloc, Dealloc, exports::___Internal___Exports___ as Exports},
  relib_shared::ModuleId,
  std::{ffi::c_void, sync::atomic::Ordering},
};

impl Exports for ModuleExportsImpl {
  fn init(
    host_owner_thread: usize,
    module: ModuleId,
    enable_alloc_tracker: bool,
    alloc: Alloc,
    dealloc: Dealloc,
  ) {
    unsafe {
      HOST_OWNER_THREAD = host_owner_thread;
      MODULE_ID = module;

      HOST_ALLOC_PROXY.alloc.set(alloc).unwrap();
      HOST_ALLOC_PROXY.dealloc.set(dealloc).unwrap();

      if enable_alloc_tracker {
        alloc_tracker::init();
      }
    }
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
      use super::thread_locals;
      unsafe {
        thread_locals::dtors::run();
      }
    }
  }

  fn spawned_threads_count() -> u64 {
    #[cfg(target_os = "linux")]
    {
      super::thread_spawn_hook::spawned_threads_count()
    }
    #[cfg(target_os = "windows")]
    {
      super::helpers::unrecoverable("spawned_threads_count called on windows")
    }
  }

  fn misc_cleanup() {
    #[cfg(target_os = "linux")]
    {
      super::mmap_hooks::cleanup();
      super::pthread_key_hooks::cleanup();
    }
    #[cfg(target_os = "windows")]
    {
      super::helpers::unrecoverable("unmap_all_mmaps called on windows")
    }
  }

  fn set_dealloc_callback(callback: *const c_void) {
    #[cfg(target_os = "windows")]
    unsafe {
      super::windows_dealloc::set_dealloc_callback(callback);
    }
    #[cfg(target_os = "linux")]
    {
      let _ = callback;
      super::helpers::unrecoverable("set_dealloc_callback called on linux")
    }
  }
}
