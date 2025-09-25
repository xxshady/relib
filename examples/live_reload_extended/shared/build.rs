use std::time::SystemTime;

fn main() {
  let key = "SHARED_CRATE_BUILD_ID";
  let value = SystemTime::now()
    .duration_since(SystemTime::UNIX_EPOCH)
    .unwrap()
    .as_millis();

  println!("cargo:rustc-env={key}={value}");
}
