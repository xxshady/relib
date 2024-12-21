use std::{env, process::Command};

pub fn provide() {
  let rust_versions = get_rust_versions();
  let lines = rust_versions.lines().collect::<Box<[&str]>>();
  let lines: &[&str] = &lines;

  let [rustc_version, _, _, _, host, _, llvm_version] = lines else {
    panic!("Unexpected rustc output");
  };
  let rustc_version = rustc_version.replace("rustc ", "");
  let host = host.replace("host: ", "");
  let llvm_version = llvm_version.replace("LLVM version: ", "");

  let workspace_version = env!("CARGO_PKG_VERSION");

  // relib_host has only "unloading" feature
  let unloading_feature = env::var("CARGO_FEATURE_UNLOADING");
  // relib_module has "unloading" and "unloading_core"
  let unloading_core_feature = env::var("CARGO_FEATURE_UNLOADING_CORE");
  let unloading_enabled = match (unloading_feature, unloading_core_feature) {
    (Ok(_), _) | (_, Ok(_)) => "1",
    (Err(_), Err(_)) => "0",
  };

  const ENV_KEY: &str = "__RELIB__CRATE_COMPILATION_INFO__";
  println!("cargo:rustc-env={ENV_KEY}={rustc_version}|{host}|{llvm_version}|{workspace_version}|{unloading_enabled}");
}

fn get_rust_versions() -> String {
  let rustc = std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_owned());
  String::from_utf8(
    Command::new(rustc)
      .arg("-v")
      .arg("-V")
      .output()
      .expect("Couldn't get rustc version")
      .stdout,
  )
  .unwrap()
}
