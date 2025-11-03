mod module;
mod errors;
pub use errors::UnloadError;
pub(crate) mod module_allocs;
pub use module_allocs::TransferToModule;
pub(crate) mod helpers;
mod imports_impl;
#[cfg(target_os = "windows")]
mod windows_dealloc;
#[cfg(target_os = "windows")]
pub(crate) mod windows_thread_spawn_hook;

relib_interface::include_exports!(gen_exports, "internal_generated_module");
relib_interface::include_imports!(gen_imports, "internal_generated_module");
pub(crate) use {
  gen_exports::ModuleExports as InternalModuleExports,
  gen_imports::init_imports as init_internal_imports,
};
