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
use helpers::{is_library_loaded, next_module_id, open_library, path_to_str, LIBRARY_LOADING_GUARD};
mod leak_library;
pub mod exports_types;
pub use exports_types::{ModuleExportsForHost, InitImports};

#[cfg(target_os = "windows")]
mod windows;

/// Loads a module (dynamic library) by specified path.
///
/// # Safety
/// This function is unsafe due to special patches related to backtraces and threads on Windows,
/// if you are on Linux ignore these safety conditions:
/// - Make sure you don't create backtraces
///   (for example, by panic or using `std::backtrace`)
///   in one thread and call this function **for the first time** from another one.
/// - Also make sure you don't spawn threads in one thread
///   and call this function **for the first time** from another one.
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
/// let module = unsafe {
///   relib_host::load_module::<()>(path_to_dylib, ())
/// };
/// let module = module.unwrap_or_else(|e| {
///   panic!("module loading failed: {e:#}");
/// });
///
/// // main function is unsafe to call (as well as any other module export) because these pre-conditions are not checked by relib:
/// // - Returned value must be actually `R` at runtime. For example if you called this function with type bool but module returns i32, UB will occur.
/// // - Type of return value must be ABI-stable.
/// // - Returned value must not be a reference-counting pointer or &'static T (see caveats on main docs page/README).
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
  unsafe {
    load_module_with_options(
      path,
      init_imports,
      #[cfg(feature = "unloading")]
      true,
    )
  }
}

/// See [`load_module`]
pub unsafe fn load_module_with_options<E: ModuleExportsForHost>(
  path: impl AsRef<OsStr>,
  init_imports: impl InitImports,

  // needs to be passed at runtime because host can load different modules with enabled and disabled alloc tracker
  #[cfg(feature = "unloading")] enable_alloc_tracker: bool,
) -> Result<Module<E>, crate::LoadError> {
  // prevent parallel loading of the same dynamic library
  // to guarantee that LoadError::ModuleAlreadyLoaded is returned
  let _loading_guard = LIBRARY_LOADING_GUARD
    .lock()
    .expect("Failed to lock library loading guard");

  #[cfg(target_os = "windows")]
  {
    windows::dbghelp::try_init_from_load_module();
    unsafe {
      #[cfg(feature = "unloading")]
      unloading::windows_thread_spawn_hook::init();
      windows::enable_hooks();
    }
  }

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
      internal_exports.init(thread_id::get(), module_id, enable_alloc_tracker);
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

  #[cfg(all(target_os = "windows", feature = "unloading"))]
  unloading::windows_thread_spawn_hook::add_module(module.library_handle);

  Ok(module)
}

/// Currently, it's only needed for Windows for backtraces (for example, `std::backtrace::Backtrace`) to work correctly in modules.
/// And it also needed for [background threads check](https://docs.rs/relib/latest/relib/docs/index.html#background-threads-check).
/// Can be called before creating any backtraces and threads if [`load_module`] panics due to already loaded `dbghelp.dll`.
///
/// **note:** This function doesn't actually do anything on Linux.
///
/// # Safety
/// Same as [`load_module`].
///
/// # Panics
/// Panics on Windows if `dbghelp.dll` was already loaded (for example, by `backtrace` crate or standard library).
pub unsafe fn init() {
  #[cfg(target_os = "windows")]
  {
    windows::dbghelp::try_init_standalone();
    unsafe {
      #[cfg(feature = "unloading")]
      unloading::windows_thread_spawn_hook::init();
      windows::enable_hooks();
    }
  }
}

/// Don't use it unless you really need to.
/// Forcibly terminates and reinitializes dbghelp.dll for backtraces on Windows.
///
/// # Safety
/// God knows...
///
#[cfg(any(target_os = "windows", relib_docs))]
#[cfg(feature = "super_special_reinit_of_dbghelp")]
pub unsafe fn forcibly_reinit_dbghelp() {
  #[cfg(target_os = "windows")]
  unsafe {
    windows::dbghelp::forcibly_reinit_dbghelp();
    windows::enable_hooks();
  }
}

// TODO: fix it
#[doc(hidden)]
#[cfg(all(target_os = "windows", feature = "unloading"))]
pub unsafe fn __suppress_unused_warning_for_linux_only_exports(
  exports: unloading::InternalModuleExports,
) {
  unsafe {
    exports.spawned_threads_count();
  }
}

#[doc(hidden)]
#[expect(unreachable_code)]
#[cfg(all(target_os = "linux", feature = "unloading"))]
pub unsafe fn __suppress_unused_warning_for_windows_only_exports(
  exports: unloading::InternalModuleExports,
) {
  unsafe { exports.set_dealloc_callback(todo!()) }
}
