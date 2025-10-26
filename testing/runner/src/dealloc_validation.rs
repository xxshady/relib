use {
  crate::helpers::{cmd, host_bin_by_directory},
  std::process::Command,
};

pub fn main() {
  let (build_debug, build_release) = cmd!(
    "cargo",
    "build",
    "--workspace",
    "--features",
    "dealloc_validation"
  );

  // invalid dealloc aborts host process thats why it should be tested in a different way
  let run_host = |directory: &str| {
    let output = Command::new(host_bin_by_directory(directory))
      .output()
      .unwrap();

    let stderr = String::from_utf8(output.stderr).unwrap();
    println!("stderr:\n{stderr}");

    assert!(
      stderr
        .contains(r#"[module id: 1] something unrecoverable happened: invalid pointer was passed to dealloc of global allocator"#)
    );
    dbg!(output.status);
    assert!(!output.status.success());
  };

  build_debug();
  run_host("debug");
  build_release();
  run_host("release");
}
