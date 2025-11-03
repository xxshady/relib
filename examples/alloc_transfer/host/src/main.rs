use {
  libloading::library_filename,
  shared::Imports,
  std::{error::Error, path::Path, process::Command, thread, time::Duration},
};

type AnyErrorResult<T = ()> = Result<T, Box<dyn Error>>;

relib_interface::include_all!();
use gen_imports::init_imports;

impl Imports for gen_imports::ModuleImportsImpl {
  fn take_drop(vec: Vec<u8>) {
    dbg!(&vec[0..10], vec.as_ptr());
    drop(vec);
  }

  fn ret() -> Vec<u8> {
    let mut vec = Vec::<u8>::with_capacity(1024 * 1024 * 100);
    for _ in 1..=(1024 * 1024 * 100) {
      vec.push(2);
    }
    vec
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
  let dylib_path = Path::new("target/debug").join(library_filename("module"));

  let module =
    unsafe { relib_host::load_module::<gen_exports::ModuleExports>(dylib_path, init_imports) }?;

  let returned = unsafe { module.call_main::<Vec<u8>>() };
  let returned = returned.unwrap();

  println!(
    "module returned vec: {:?}",
    &returned[..10.min(returned.len())]
  );

  let ret_vec = unsafe {
    let vec = vec![1, 2, 3, 4, 1, 2, 3, 4, 5, 1, 2, 3, 4, 5];
    module.exports().take_forget(vec).unwrap();

    let vec = vec![1, 2, 3, 4, 5, 1, 2, 3, 4, 5];
    module.exports().take_drop(vec).unwrap();

    module.exports().ret().unwrap()
  };

  // when unloading fails it is not safe to load it again
  module
    .unload()
    .map_err(|e| format!("module unloading failed: {e:#}"))?;

  dbg!();

  // unsafe {
  //   let _ = std::alloc::alloc_zeroed(std::alloc::Layout::new::<[u8; 1024 * 1024 * 100]>());
  // }

  dbg!();

  println!(
    "after unloading module returned vec: {:?} ret_vec: {:?}",
    &returned[..10.min(returned.len())],
    &ret_vec[..10.min(ret_vec.len())]
  );

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
