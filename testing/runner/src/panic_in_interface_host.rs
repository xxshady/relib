use std::process::Command;

use crate::helpers::cmd;

pub fn main() {
  let (build_debug, build_release) = cmd!(
    "cargo",
    "build",
    "--workspace",
    "--features",
    "panic_in_interface_host"
  );

  // panic in host aborts host process thats why it should be tested in a different way
  let run_host = |directory: &str| {
    let output = Command::new(format!("target/{directory}/test_host"))
      .output()
      .unwrap();

    let stderr = String::from_utf8(output.stderr).unwrap();
    println!("stderr:\n{stderr}");

    assert!(stderr
      .contains(r#"[relib] host panicked while executing import "panic" of module, aborting"#));
    dbg!(output.status);
    assert!(!output.status.success());
  };

  build_debug();
  run_host("debug");
  build_release();
  run_host("release");
}
