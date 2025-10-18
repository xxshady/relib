use std::{error::Error, fs, process::Command, thread, time::Duration};

type AnyErrorResult<T = ()> = Result<T, Box<dyn Error>>;

relib_interface::include_exports!();
use gen_exports::ModuleExports;

relib_interface::include_imports!();
use gen_imports::{init_imports, ModuleImportsImpl};
use shared::imports::Imports;

impl Imports for ModuleImportsImpl {
  fn foo() -> i32 {
    123
  }
}

fn main() {
  if let Err(e) = run_host() {
    panic!("{e:#}");
  }
}

fn run_host() -> AnyErrorResult {
  run_module()?;

  let mut build_failed_in_prev_iteration = false;
  loop {
    match build_module()? {
      BuildResult::Success => {
        // inserting new line for more clear output of module after compilation failures or previous runs of the module
        println!();
        run_module()?;

        build_failed_in_prev_iteration = false;
      }
      BuildResult::Failure(message) => {
        if build_failed_in_prev_iteration {
          continue;
        }
        build_failed_in_prev_iteration = true;

        println!("failed to build the module:\n{message}");
      }
      BuildResult::NoChange => {}
    }
    thread::sleep(Duration::from_millis(50));
  }
}

fn run_module() -> AnyErrorResult {
  let file_name = if cfg!(windows) {
    "module.dll"
  } else {
    "libmodule.so"
  };
  let path_to_dylib = format!("target/debug/{file_name}");
  let copy_path_to_dylib = format!("target/debug/copy_{file_name}");

  fs::copy(&path_to_dylib, &copy_path_to_dylib)?;

  let module = unsafe { relib_host::load_module::<ModuleExports>(path_to_dylib, init_imports) }?;

  let module_shared_build_id = unsafe { module.exports().shared_build_id() }.unwrap();
  let host_shared_build_id = shared::build_id();

  // when shared crate is modified it's no longer safe to load the module,
  // so we need to stop here
  if module_shared_build_id != host_shared_build_id {
    return Err(
      format!(
        "shared crate was modified, module potentially contains incompatible code\n\
        shared build id of:\n\
        host:   {}\n\
        module: {}",
        host_shared_build_id, module_shared_build_id
      )
      .into(),
    );
  }

  let returned = unsafe { module.call_main::<()>() };
  if returned.is_none() {
    println!("module panicked");
  }

  // when unloading fails it is not safe to load it again
  module
    .unload()
    .map_err(|e| format!("module unloading failed: {e:#}"))?;

  Ok(())
}

fn build_module() -> AnyErrorResult<BuildResult> {
  let output = Command::new("cargo").args(["build"]).output()?;
  let stderr = String::from_utf8(output.stderr)?;

  if !output.status.success() {
    return Ok(BuildResult::Failure(stderr));
  }

  Ok(if stderr.contains("Compiling") {
    BuildResult::Success
  } else {
    BuildResult::NoChange
  })
}

enum BuildResult {
  Success,
  Failure(String),
  NoChange,
}
