mod helpers;
use helpers::{call_host_by_directory, cmd};
mod code_change;
mod multiple_modules;
mod panic_in_interface_host;
mod backtrace_unloading_host_as_dylib;

const TEST_FEATURES: &[&str] = &[
  #[cfg(target_os = "windows")]
  "dbghelp_is_already_loaded_init",
  #[cfg(target_os = "windows")]
  "dbghelp_is_already_loaded_panic",
  "is_already_loaded_error",
  "backtrace_unloading",
  "unloading",
  "no_unloading",
  "exportify",
  "exportify,ret_primitive_main",
  "exportify,ret_heap_main",
  "exportify,panic_main",
  #[cfg(target_os = "linux")]
  "threads_check",
  "before_unload_panic",
  // panic_in_interface_host is in its own module
  "panic_in_interface_module",
];

fn main() {
  // test_features("debug");
  // test_features("release");

  // multiple_modules::main();
  code_change::main();
  // panic_in_interface_host::main();

  // backtrace_unloading_host_as_dylib::main();

  println!();
  println!();
  println!("all tests successfully executed");
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
