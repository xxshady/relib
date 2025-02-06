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
use helpers::{is_library_loaded, next_module_id, open_library, path_to_str};
mod leak_library;
pub mod exports_types;
pub use exports_types::{ModuleExportsForHost, InitImports};

#[cfg(target_os = "windows")]
mod windows;

/// Loads a module (dynamic library) by specified path.
///
/// # Safety
/// This function is unsafe due to special patches related to backtraces on Windows,
/// if you are on Linux ignore this safety condition:
/// - Make sure you don't create backtraces
///   (for example, by panic or using `std::backtrace`)
///   in one thread and call this function **for the first time** from another one.
///
/// If you can't guarantee it when you call this function consider using
/// [`init`] at the start of your program.
///
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
/// // - Returned value must not be a reference-counting pointer (see limitations in README or docs page).
/// let returned_value = unsafe { module.call_main::<()>() };
///
/// // if module panics while executing any export it returns None
/// // (panic will be printed by module)
/// if returned_value.is_none() {
///   println!("module panicked");
/// }
/// ```
///
/// # Panics
/// Panics on Windows if `dbghelp.dll` was already loaded. See [`init`]
pub unsafe fn load_module<E: ModuleExportsForHost>(
  path: impl AsRef<OsStr>,
  init_imports: impl InitImports,
) -> Result<Module<E>, crate::LoadError> {
  #[cfg(target_os = "windows")]
  windows::dbghelp::try_init_from_load_module();

  let path = Path::new(path.as_ref());
  let path_str = path_to_str(path);

  if is_library_loaded(path_str) {
    return Err(LoadError::ModuleAlreadyLoaded);
  }

  let library = open_library(path)?;

  let module_comp_info = unsafe {
    let compiled_with = library.get(b"__RELIB__CRATE_COMPILATION_INFO__\0");
    let Ok(compiled_with) = compiled_with else {
      return Err(LoadError::CouldNotGetCompilationInfo);
    };
    let compiled_with: Symbol<*const Str> = compiled_with;
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

  #[cfg(target_os = "windows")]
  windows::dbghelp::add_module(path_str);

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

/// Currently, it's only needed for backtraces (for example, `std::backtrace::Backtrace`) to work correctly in modules on Windows.
/// Doesn't actually do anything on Linux.
/// Can be called before creating any backtraces if [`load_module`] panics due to already loaded `dbghelp.dll`.
///
/// # Safety
/// Same as [`load_module`].
///
/// # Panics
/// Panics on Windows if `dbghelp.dll` was already loaded (for example, by `backtrace` crate or standard library).
pub unsafe fn init() {
  #[cfg(target_os = "windows")]
  windows::dbghelp::try_init();
}

// TODO: fix it
#[cfg(all(target_os = "windows", feature = "unloading"))]
#[expect(clippy::missing_safety_doc)]
pub unsafe fn __suppress_unused_warning_for_linux_only_exports(
  exports: unloading::InternalModuleExports,
) {
  exports.spawned_threads_count();
}

#[cfg(all(target_os = "linux", feature = "unloading"))]
#[expect(clippy::missing_safety_doc)]
pub unsafe fn __suppress_unused_warning_for_windows_only_exports(
  exports: unloading::InternalModuleExports,
) {
  #[expect(unreachable_code)]
  exports.set_dealloc_callback(todo!());
}
