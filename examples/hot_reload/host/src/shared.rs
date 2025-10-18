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

// TODO: build all at the same place?
pub fn build_module(name: &str) -> AnyErrorResult<BuildResult> {
  let output = measure_time("cargo build", || {
    Command::new("cargo").arg("build").output()
  })?;

  let stderr = String::from_utf8(output.stderr)?;

  if !output.status.success() {
    return Ok(BuildResult::Failure(stderr));
  }

  Ok(if stderr.contains(&format!("Compiling {name}")) {
    BuildResult::Success
  } else {
    BuildResult::NoChange
  })
}

pub enum BuildResult {
  Success,
  Failure(String),
  NoChange,
}

pub fn measure_time<R, F: FnOnce() -> R>(_label: &str, f: F) -> R {
  // if label == "cargo build" {
  //   let start = Instant::now();
  //   let result = f();
  //   let duration = start.elapsed();
  //   println!("{label} took {duration:?}");
  //   result
  // } else {
  //   f()
  // }
  f()
}

pub fn load_module<E: ModuleExportsForHost>(
  name: &str,
  init_imports: impl InitImports,
) -> AnyErrorResult<Module<E>> {
  let file_name = if cfg!(windows) {
    format!("{name}.dll")
  } else {
    format!("lib{name}.so")
  };
  let path_to_dylib = format!("target/debug/{file_name}");
  let copy_path_to_dylib = format!("target/debug/copy_{file_name}");

  fs::copy(&path_to_dylib, &copy_path_to_dylib)?;

  let module = unsafe { relib_host::load_module::<E>(copy_path_to_dylib, init_imports)? };

  Ok(module)
}
