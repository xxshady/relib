use std::{error::Error, process::Command, thread, time::Duration};

type AnyErrorResult<T = ()> = Result<T, Box<dyn Error>>;

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
  let path_to_dylib = "target/debug/".to_owned() + file_name;

  let module = relib_host::load_module::<()>(path_to_dylib, ())?;

  let returned = unsafe { module.call_main::<()>() };
  if returned.is_none() {
    println!("module panicked");
  }

  // when unloading fails it is not safe to load it again
  module
    .unload()
    .or_else(|e| Err(format!("module unloading failed: {e:#}")))?;

  Ok(())
}

fn build_module() -> AnyErrorResult<BuildResult> {
  let output = Command::new("cargo")
    .args(["build", "--package", "module"])
    .output()?;
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
