mod compilation_info;

#[cfg(feature = "unloading_core")]
mod unloading_core;
#[cfg(feature = "unloading_core")]
pub use unloading_core::AllocTracker;

#[cfg(all(feature = "global_alloc_tracker", not(feature = "unloading_core")))]
compile_error!(
  "\"global_alloc_tracker\" feature cannot be enabled without \"unloading_core\" feature:\n\
  you can either: \
  enable \"unloading\" feature or \
  disable \"global_alloc_tracker\" and enable \"unloading_core\""
);

// TODO: add support for conditional compilation in relib_interface
#[doc(hidden)]
#[cfg(all(feature = "unloading_core", not(feature = "dealloc_validation")))]
pub use unloading_core::_suppress_warn;

#[cfg(not(feature = "unloading_core"))]
mod no_unloading;

mod host_alloc_proxy;
pub use {host_alloc_proxy::HostAllocProxy, relib_export::export};

#[doc(hidden)]
pub mod __internal {
  #[cfg(feature = "unloading_core")]
  use relib_shared::TransferTarget;

  #[cfg(feature = "unloading_core")]
  pub fn module_id() -> relib_shared::ModuleId {
    unsafe { crate::unloading_core::MODULE_ID }
  }

  pub struct TransferToHost;

  unsafe impl TransferTarget for TransferToHost {
    type ExtraContext = ();

    fn transfer(ptr: *mut u8, _: &()) {
      #[cfg(feature = "unloading_core")]
      crate::unloading_core::alloc_tracker::transfer_alloc_to_host(ptr);

      #[cfg(not(feature = "unloading_core"))]
      let _ = ptr;
    }
  }
}
