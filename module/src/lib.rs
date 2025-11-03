mod compilation_info;

pub use relib_export::export;

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

#[doc(hidden)]
pub mod __internal {
  #[cfg(feature = "unloading_core")]
  pub use crate::unloading_core::TransferToHost;

  pub fn module_id() -> relib_shared::ModuleId {
    unsafe { crate::unloading_core::MODULE_ID }
  }
}
