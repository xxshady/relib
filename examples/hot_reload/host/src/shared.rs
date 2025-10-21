use std::{
  cell::RefCell,
  error::Error,
  fs,
  process::Command,
  rc::Rc,
  thread,
  time::{Duration, Instant},
};
use relib_host::{InitImports, Module, ModuleExportsForHost};

pub type AnyErrorResult<T = ()> = anyhow::Result<T>;

pub fn load_module<E: ModuleExportsForHost>(
  name: &str,
  init_imports: impl InitImports,
  enable_alloc_tracker: bool,
) -> AnyErrorResult<Module<E>> {
  let file_name = if cfg!(windows) {
    format!("{name}.dll")
  } else {
    format!("lib{name}.so")
  };
  let path_to_dylib = format!("target/debug/{file_name}");
  let copy_path_to_dylib = format!("target/debug/copy_{file_name}");

  fs::copy(&path_to_dylib, &copy_path_to_dylib)?;

  let module = unsafe {
    relib_host::load_module_with_options::<E>(
      copy_path_to_dylib,
      init_imports,
      enable_alloc_tracker,
    )?
  };

  Ok(module)
}
