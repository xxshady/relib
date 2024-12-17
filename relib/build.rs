use std::{env, fs, path::Path};

fn main() {
  let out_dir = env::var("OUT_DIR").unwrap();
  let out_dir = Path::new(&out_dir);

  let docs = out_dir.join("docs.md");
  fs::copy("docs.md", docs).unwrap();

  let readme = out_dir.join("README.md");
  fs::copy("../README.md", readme).unwrap();
}
