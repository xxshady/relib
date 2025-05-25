mod module;
mod errors;
pub use errors::UnloadError;
pub(crate) mod module_allocs;
pub(crate) mod helpers;
mod imports_impl;
#[cfg(target_os = "windows")]
mod windows_dealloc;
#[cfg(target_os = "windows")]
pub(crate) mod windows_thread_spawn_hook;

relib_interface::include_exports!();
relib_interface::include_imports!();
pub(crate) use gen_exports::ModuleExports as InternalModuleExports;
pub(crate) use gen_imports::init_imports as init_internal_imports;
