use std::{
  io::{Read, Write},
  process::{Command, Stdio},
  thread,
  time::Duration,
};

use crate::helpers::{cmd, host_bin_by_directory};

pub fn main() {
  let (build_debug, build_release) =
    cmd!("cargo", "build", "--workspace", "--features", "code_change");

  build_debug();
  run_host("debug");
  build_release();
  run_host("release");
}

fn run_host(directory: &str) {
  let mut host_proc = Command::new(host_bin_by_directory(directory))
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .stdin(Stdio::piped())
    .spawn()
    .unwrap();

  let mut stdout = host_proc.stdout.take().unwrap();
  let mut stderr = host_proc.stderr.take().unwrap();
  let mut stdin = host_proc.stdin.take().unwrap();

  let (rebuild_debug, rebuild_release) = cmd!(
    "cargo",
    "build",
    "--workspace",
    "--features",
    "code_change,code_change_before_unload"
  );

  if directory == "release" {
    rebuild_release();
  } else {
    rebuild_debug();
  }

  stdin.write_all(b"next\n").unwrap();

  let (rebuild_debug, rebuild_release) = cmd!(
    "cargo",
    "build",
    "--workspace",
    "--features",
    "code_change,code_change_before_unload,code_change_leak"
  );

  if directory == "release" {
    rebuild_release();
  } else {
    rebuild_debug();
  }

  stdin.write_all(b"next\n").unwrap();
  thread::sleep(Duration::from_millis(500));
  stdin.write_all(b"end\n").unwrap();

  let status = host_proc.wait().unwrap();

  let mut stdout_content = String::new();
  stdout.read_to_string(&mut stdout_content).unwrap();
  let mut stderr_content = String::new();
  stderr.read_to_string(&mut stderr_content).unwrap();

  println!(
    "host code_change test output\n\
    stdout:\n\
    {stdout_content}\n\
    stderr:\n\
    {stderr_content}"
  );

  assert!(status.success());
  assert!(stdout_content.contains("[module] before_unload"));
}
