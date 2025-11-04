use {
  relib_shared::ModuleId,
  std::sync::atomic::{AtomicBool, Ordering},
};

relib_interface::include_exports!(gen_exports, "internal_generated_module");
relib_interface::include_imports!(gen_imports, "internal_generated_module");

#[cfg(target_os = "linux")]
mod thread_locals;
#[cfg(target_os = "linux")]
mod thread_spawn_hook;
/// hooks of libc mmap64 and munmap to cleanup
/// leaked memory mappings on module unloading
/// (for example, std backtrace leaks them)
#[cfg(target_os = "linux")]
mod mmap_hooks;
#[cfg(target_os = "linux")]
mod pthread_key_hooks;
mod helpers;
mod exports_impl;

pub mod alloc_tracker;
#[cfg(all(feature = "unloading_core", not(feature = "dealloc_validation")))]
pub use alloc_tracker::_suppress_warn;
pub use alloc_tracker::AllocTracker;

#[cfg(target_os = "windows")]
mod windows_dll_main;
#[cfg(target_os = "windows")]
mod windows_dealloc;

/// Middleware for tracking all allocations to deallocate leaks
/// (for example `std::mem:forget`, static items) on module unload.
/// It sends all allocations and deallocations to host because to
/// store allocations we need to allocate unknown amount of memory.
///
/// **Safety requirement** if you want to set your own `#[global_allocator]`:
/// it must use global allocator of the host (you can use [`HostAllocProxy`] for it).
#[cfg(feature = "global_alloc_tracker")]
mod __global_alloc {
  use crate::{host_alloc_proxy::HostAllocProxy, unloading_core::AllocTracker};

  #[global_allocator]
  static ALLOC_TRACKER: AllocTracker<HostAllocProxy> = AllocTracker::new(HostAllocProxy);
}

static ALLOCATOR_LOCK: AtomicBool = AtomicBool::new(false);
fn allocator_lock() -> bool {
  ALLOCATOR_LOCK.load(Ordering::SeqCst)
}

// SAFETY: will be initialized on one thread once and then never change
pub static mut MODULE_ID: ModuleId = 0;

// The id of the thread in which this module was loaded and in which it must be unloaded
//
// SAFETY: will be initialized on one thread once and then never change
static mut HOST_OWNER_THREAD: usize = 0;
