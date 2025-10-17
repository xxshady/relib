use std::{env, path::Path, time::SystemTime};

use cargo_metadata::{CargoOpt, MetadataCommand};

fn main() {
  dbg!();

  let key = "MAIN_CONTRACT_CRATE_BUILD_ID";
  let value = SystemTime::now()
    .duration_since(SystemTime::UNIX_EPOCH)
    .unwrap()
    .as_millis();

  println!("cargo:rustc-env={key}={value}");

  rerun_if_local_dependencies_change();
}

fn rerun_if_local_dependencies_change() {
  // Get metadata for the current workspace
  let metadata = MetadataCommand::new()
    .features(CargoOpt::AllFeatures)
    .exec()
    .expect("Failed to execute cargo metadata");

  // Find the current package in the workspace
  let pkg_name = env::var("CARGO_PKG_NAME").unwrap();
  let current_pkg = metadata
    .workspace_packages()
    .into_iter()
    .find(|p| p.name == pkg_name)
    .unwrap();

  // Resolve the dependency graph for the current package
  let pkg_graph = metadata
    .resolve
    .as_ref()
    .unwrap()
    .nodes
    .iter()
    .find(|n| n.id == current_pkg.id)
    .unwrap();

  // Iterate over dependencies, find local ones, and add rerun-if-changed
  for dep_id in dbg!(&pkg_graph.dependencies) {
    let Some(dep_pkg) = metadata.packages.iter().find(|p| p.id == *dep_id) else {
      continue;
    };
    // We only need local dependencies
    if dep_pkg.source.is_some() {
      continue;
    }

    // The manifest_path is the path to Cargo.toml, so we get its parent directory.
    let dep_dir = Path::new(&dep_pkg.manifest_path).parent().unwrap();
    println!("cargo:rerun-if-changed={}", dep_dir.to_str().unwrap());
  }
}
