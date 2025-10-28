use {
  crate::helpers::{call_host_by_directory, cmd},
  std::fs,
};

pub fn main() {
  // ------------------------- windows_background_threads
  let (build_debug, build_release) = cmd!(
    "cargo",
    "build",
    "--workspace",
    "--features",
    "windows_background_threads"
  );

  build_debug();
  run_modules("debug");
  build_release();
  run_modules("release");

  // ------------------------- windows_background_threads_fail
  let (build_debug, build_release) = cmd!(
    "cargo",
    "build",
    "--workspace",
    "--features",
    "windows_background_threads_fail"
  );
  build_debug();
  call_host_by_directory("debug");
  build_release();
  call_host_by_directory("release");
}

fn run_modules(directory: &str) {
  for idx in 0..2 {
    fs::copy(
      format!("target/{directory}/test_module.dll"),
      format!("target/{directory}/windows_background_threads__test_module_{idx}.dll"),
    )
    .unwrap();
  }
  call_host_by_directory(directory);
}
