use {
  crate::helpers::{call_host_by_directory, cmd},
  std::{
    env::consts::{DLL_PREFIX, DLL_SUFFIX},
    fs,
    path::Path,
  },
};

pub fn main() {
  let (build_debug, build_release) = cmd!(
    "cargo",
    "build",
    "--workspace",
    "--features",
    "multiple_modules"
  );

  build_debug();
  run_multiple_modules("debug");
  build_release();
  run_multiple_modules("release");
}

fn run_multiple_modules(directory: &str) {
  let dylib_filename = format!("{}test_module{}", DLL_PREFIX, DLL_SUFFIX);
  let target_directory = Path::new("target").join(directory);
  let dylib_path = target_directory.join(dylib_filename);

  for idx in 0..10 {
    let dylib_copy_filename = format!("{}test_module_{idx}{}", DLL_PREFIX, DLL_SUFFIX);
    let dylib_copy_path = target_directory.join(dylib_copy_filename);
    fs::copy(&dylib_path, dylib_copy_path).unwrap();
  }
  call_host_by_directory(directory);
}
