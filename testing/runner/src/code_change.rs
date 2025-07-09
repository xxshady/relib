use std::{
  io::{Read, Write},
  process::{ChildStderr, Command, Stdio},
  sync::atomic::{AtomicI32, Ordering::Relaxed},
  thread,
  time::Duration,
};

use crate::helpers::{cmd, host_bin_by_directory};

pub fn main() {
  let (build_debug, build_release) =
    cmd!("cargo", "build", "--workspace", "--features", "code_change");

  build_debug();
  run_host("debug");
  reset_iteration();

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

  let mut stderr_content = String::new();

  // try blocks when
  let res: Result<(), ()> = (|| {
    wait_for_end_of_exec(&mut stderr, &mut stderr_content)?;

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
    wait_for_end_of_exec(&mut stderr, &mut stderr_content)?;

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
    wait_for_end_of_exec(&mut stderr, &mut stderr_content)?;

    let (rebuild_debug, rebuild_release) = cmd!(
      "cargo",
      "build",
      "--workspace",
      "--features",
      "code_change,code_change_before_unload,code_change_leak,code_change_backtrace_unloading"
    );

    if directory == "release" {
      rebuild_release();
    } else {
      rebuild_debug();
    }

    for _ in 1..=10 {
      stdin.write_all(b"next\n").unwrap();
      wait_for_end_of_exec(&mut stderr, &mut stderr_content)?;
    }

    // TODO: add assert with memory usage check
    let (rebuild_debug, rebuild_release) = cmd!(
      "cargo",
      "build",
      "--workspace",
      "--features",
      "code_change,code_change_before_unload,code_change_leak,code_change_backtrace_unloading,code_change_backtrace_unloading2"
    );

    if directory == "release" {
      rebuild_release();
    } else {
      rebuild_debug();
    }

    for _ in 1..=10 {
      stdin.write_all(b"next\n").unwrap();
      wait_for_end_of_exec(&mut stderr, &mut stderr_content)?;
    }

    thread::sleep(Duration::from_millis(500));
    stdin.write_all(b"end\n").unwrap();

    Ok(())
  })();

  dbg!(&res);

  let status = host_proc.wait().unwrap();

  let mut stdout_content = String::new();
  stdout.read_to_string(&mut stdout_content).unwrap();
  stderr.read_to_string(&mut stderr_content).unwrap();

  if res.is_ok() {
    assert!(stderr_content.contains("received_end_______________\n"));
  }

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

static ITERATION: AtomicI32 = AtomicI32::new(0);

fn reset_iteration() {
  ITERATION.store(0, Relaxed);
}

fn wait_for_end_of_exec(stderr: &mut ChildStderr, stderr_content: &mut String) -> Result<(), ()> {
  let i = {
    let prev = ITERATION.load(Relaxed);
    let next = prev + 1;
    ITERATION.store(next, Relaxed);
    next
  };

  let expected_message = format!("code_change_module_has_been_exec_{i}\n");

  println!("waiting for {expected_message}");

  let mut buf = vec![0_u8; 1000];
  loop {
    let count = stderr.read(&mut buf).unwrap();
    if count == 0 {
      return Err(());
    }

    let received_chunk = std::str::from_utf8(&buf[..count]).unwrap();
    *stderr_content += received_chunk;
    dbg!(received_chunk);

    if received_chunk.contains(&expected_message) {
      break;
    }
  }

  Ok(())
}
