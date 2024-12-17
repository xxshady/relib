use std::{ffi::OsStr, path::Path};

use libloading::Symbol;

use relib_internal_shared::Str;

relib_interface::include_exports!();
relib_interface::include_imports!();
use gen_exports::ModuleExports as InternalModuleExports;
use gen_imports::init_imports as init_internal_imports;

mod errors;
pub use errors::Error;
mod module;
pub use module::Module;
mod module_allocs;
mod helpers;
use helpers::{next_module_id, open_library};
mod imports_impl;
mod leak_library;
pub mod exports_types;
pub use exports_types::{ModuleExportsForHost, InitImports, ModuleValue};

/// # Example
/// ```
/// // replace "?" with your file name, for example if you named module crate as "module"
/// // on linux the path will be "target/debug/libmodule.so", on windows it will be "target/debug/module.dll"
/// let path_to_dylib = "target/debug/?";
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
) -> Result<Module<E>, crate::Error> {
  let path = Path::new(path.as_ref());

  #[cfg(target_os = "linux")]
  {
    use helpers::linux::is_library_loaded;

    if is_library_loaded(path) {
      return Err(Error::ModuleAlreadyLoaded);
    }
  }

  let library = open_library(path)?;

  let compiled_with = unsafe {
    let compiled_with: Symbol<*const Str> = library.get(b"__RELIB__CRATE_COMPILATION_INFO__\0")?;
    let compiled_with: &Str = &**compiled_with;
    compiled_with.to_string()
  };

  let expected = crate_compilation_info::get!();
  if compiled_with != expected {
    return Err(Error::ModuleCompilationMismatch(
      compiled_with,
      expected.to_owned(),
    ));
  }

  init_internal_imports(&library);

  let module_id = next_module_id();

  module_allocs::add_module(module_id);

  let internal_exports = InternalModuleExports::new(&library);
  unsafe {
    internal_exports.init(thread_id::get(), module_id);
  }

  let pub_exports = E::new(&library);
  init_imports.init(&library);

  let module = Module::new(
    module_id,
    library,
    internal_exports,
    pub_exports,
    path.to_owned(),
  );
  Ok(module)
}

// TODO: fix it
#[cfg(target_os = "windows")]
#[expect(clippy::missing_safety_doc)]
pub unsafe fn __suppress_unused_warning_for_linux_only_exports(exports: InternalModuleExports) {
  exports.spawned_threads_count();
}
