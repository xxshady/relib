use std::{ffi::OsStr, path::Path};

use libloading::Symbol;

use relib_internal_shared::Str;

mod errors;
pub use errors::LoadError;

#[cfg(feature = "unloading")]
mod unloading;
#[cfg(feature = "unloading")]
pub use unloading::*;

mod module;
pub use module::Module;
mod helpers;
use helpers::{next_module_id, open_library};
mod leak_library;
pub mod exports_types;
pub use exports_types::{ModuleExportsForHost, InitImports, ModuleValue};

/// # Example
/// ```
/// let path_to_dylib = if cfg!(target_os = "linux") {
///   "target/debug/libmodule.so"
/// } else {
///   "target/debug/module.dll"
/// };
///
/// // `()` means empty imports and exports, module doesn't import or export anything
/// let module = relib_host::load_module::<()>(path_to_dylib, ()).unwrap();
///
/// // main function is unsafe to call (as well as any other module export) because these pre-conditions are not checked by relib:
/// // - Returned value must be actually `R` at runtime. For example if you called this function with type bool but module returns i32, UB will occur.
/// // - Type of return value must be FFI-safe.
/// let returned_value = unsafe { module.call_main::<()>() };
///
/// // if module panics while executing any export it returns None
/// // (panic will be printed by module)
/// if returned_value.is_none() {
///   println!("module panicked");
/// }
/// ```
pub fn load_module<E: ModuleExportsForHost>(
  path: impl AsRef<OsStr>,
  init_imports: impl InitImports,
) -> Result<Module<E>, crate::LoadError> {
  let path = Path::new(path.as_ref());

  #[cfg(target_os = "linux")]
  {
    use helpers::linux::is_library_loaded;

    if is_library_loaded(path) {
      return Err(LoadError::ModuleAlreadyLoaded);
    }
  }

  let library = open_library(path)?;

  let module_comp_info = unsafe {
    let compiled_with: Symbol<*const Str> = library.get(b"__RELIB__CRATE_COMPILATION_INFO__\0")?;
    let compiled_with: &Str = &**compiled_with;
    compiled_with.to_string()
  };

  let host_comp_info = relib_internal_crate_compilation_info::get!();
  if module_comp_info != host_comp_info {
    return Err(LoadError::ModuleCompilationMismatch {
      module: module_comp_info,
      host: host_comp_info.to_owned(),
    });
  }

  let module_id = next_module_id();

  #[cfg(feature = "unloading")]
  let internal_exports = {
    unloading::init_internal_imports(&library);
    unloading::module_allocs::add_module(module_id);

    let internal_exports = unloading::InternalModuleExports::new(&library);
    unsafe {
      internal_exports.init(thread_id::get(), module_id);
    }
    internal_exports
  };

  let pub_exports = E::new(&library);
  init_imports.init(&library);

  let module = Module::new(
    module_id,
    library,
    pub_exports,
    #[cfg(feature = "unloading")]
    (internal_exports, path.to_owned()),
  );
  Ok(module)
}

// TODO: fix it
#[cfg(all(target_os = "windows", feature = "unloading"))]
#[expect(clippy::missing_safety_doc)]
pub unsafe fn __suppress_unused_warning_for_linux_only_exports(
  exports: unloading::InternalModuleExports,
) {
  exports.spawned_threads_count();
}
