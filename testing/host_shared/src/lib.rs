use std::env::consts::{DLL_PREFIX, DLL_SUFFIX};

use relib_host::{InitImports, LoadError, Module, ModuleExportsForHost};

pub fn load_module<Exports: ModuleExportsForHost, MainRet: Clone>(
  init_imports: impl InitImports,
  check_panic: bool,
) -> (Module<Exports>, Option<MainRet>) {
  load_module_with_name(init_imports, "test_module", check_panic)
}

pub fn load_module_with_name<Exports: ModuleExportsForHost, MainRet: Clone>(
  init_imports: impl InitImports,
  name: &str,
  check_panic: bool,
) -> (Module<Exports>, Option<MainRet>) {
  let path = dylib_filename(name);
  load_module_with_path(init_imports, &path, check_panic)
}

pub fn load_module_with_result<Exports: ModuleExportsForHost, MainRet: Clone>(
  init_imports: impl InitImports,
  check_panic: bool,
) -> Result<(Module<Exports>, Option<MainRet>), LoadError> {
  let path = dylib_filename("test_module");
  load_module_with_path_and_result(init_imports, &path, check_panic)
}

pub fn load_module_with_path<Exports: ModuleExportsForHost, MainRet: Clone>(
  init_imports: impl InitImports,
  path: &str,
  check_panic: bool,
) -> (Module<Exports>, Option<MainRet>) {
  load_module_with_path_and_result(init_imports, path, check_panic).unwrap_or_else(|e| {
    panic!(
      "load_module_with_path_and_result path: {path} failed:\n\
      {e:#}"
    );
  })
}

pub fn load_module_with_path_and_result<Exports: ModuleExportsForHost, MainRet: Clone>(
  init_imports: impl InitImports,
  path: &str,
  check_panic: bool,
) -> Result<(Module<Exports>, Option<MainRet>), LoadError> {
  let target_dir = current_target_dir();
  let path = format!("{target_dir}/{path}");

  let module = unsafe { relib_host::load_module::<Exports>(path, init_imports) }?;

  let ret = unsafe { module.call_main::<MainRet>() };

  if check_panic {
    assert!(ret.is_some(), "module main fn panicked");
  }

  Ok((module, ret))
}

pub fn current_target_dir() -> &'static str {
  if cfg!(debug_assertions) {
    "target/debug"
  } else {
    "target/release"
  }
}

// TODO: use libloading::library_filename?
pub fn dylib_filename(name: &str) -> String {
  format!("{DLL_PREFIX}{name}{DLL_SUFFIX}")
}
