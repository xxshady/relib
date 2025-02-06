use std::fs;

use crate::{
  cmd,
  helpers::{call_host_by_directory, dylib_filename},
};

pub fn main() {
  let (build_debug, build_release) = cmd!(
    "cargo",
    "build",
    "--workspace",
    "--features",
    "backtrace_unloading_host_as_dylib"
  );

  build_debug();
  {
    let filename = dylib_filename("test_module");
    let dir = "target/debug";
    let from = format!("{dir}/{filename}");
    let to_dir = format!("{dir}/backtrace_unloading_host_as_dylib");
    let to = format!("{to_dir}/{filename}");
    fs::create_dir_all(&to_dir).unwrap();
    fs::copy(&from, &to).unwrap_or_else(|e| {
      panic!(
        "copy\n\
        | {from}\n\
        -> {to}\n\
        failed: {e}"
      );
    });
  }
  call_host_by_directory("debug");

  build_release();
  {
    let filename = dylib_filename("test_module");
    let dir = "target/release";
    let from = format!("{dir}/{filename}");
    let to_dir = format!("{dir}/backtrace_unloading_host_as_dylib");
    let to = format!("{to_dir}/{filename}");
    fs::create_dir_all(&to_dir).unwrap();
    fs::copy(&from, &to).unwrap_or_else(|e| {
      panic!(
        "copy\n\
        | {from}\n\
        -> {to}\n\
        failed: {e}"
      );
    });
  }
  call_host_by_directory("release");
}
