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
