use std::fs;

use crate::helpers::{call_host_by_directory, cmd};

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
  for idx in 0..10 {
    if cfg!(target_os = "linux") {
      fs::copy(
        format!("target/{directory}/libtest_module.so"),
        format!("target/{directory}/libtest_module_{idx}.so"),
      )
      .unwrap();
    } else {
      fs::copy(
        format!("target/{directory}/test_module.dll"),
        format!("target/{directory}/test_module_{idx}.dll"),
      )
      .unwrap();
    }
  }
  call_host_by_directory(directory);
}
