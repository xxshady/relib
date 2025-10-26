mod compilation_info;

pub use relib_export::export;

#[cfg(feature = "unloading_core")]
mod unloading_core;
#[cfg(feature = "unloading_core")]
pub use unloading_core::*;

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
