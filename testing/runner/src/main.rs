mod helpers;
use helpers::{call_host_by_directory, cmd};
mod code_change;
mod multiple_modules;
mod panic_in_interface_host;

const TEST_FEATURES: &[&str] = &[
  "unloading",
  "no_unloading",
  "exportify",
  "exportify,ret_primitive_main",
  "exportify,ret_heap_main",
  "exportify,panic_main",
  "threads_check",
  "before_unload_panic",
  // panic_in_interface_host is in its own module
  "panic_in_interface_module",
];

fn main() {
  test_features("debug");
  test_features("release");

  multiple_modules::main();
  code_change::main();
  panic_in_interface_host::main();
}

fn test_features(directory: &str) {
  for feature in TEST_FEATURES {
    let (build_debug, build_release) = cmd!("cargo", "build", "--workspace", "--features", feature);

    if directory == "release" {
      build_release();
    } else {
      build_debug();
    }

    call_host_by_directory(directory);
  }
}
