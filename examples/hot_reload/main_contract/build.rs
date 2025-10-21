use std::{collections::HashSet, env, path::Path, time::SystemTime};
use cargo_metadata::{CargoOpt, Metadata, MetadataCommand, PackageId};

fn main() {
  let key = "MAIN_CONTRACT_CRATE_BUILD_ID";
  let value = SystemTime::now()
    .duration_since(SystemTime::UNIX_EPOCH)
    .unwrap()
    .as_millis();

  // TODO: it's probably not needed anymore
  // println!("cargo:rustc-env={key}={value}");
  println!("cargo:rustc-env={key}=1");

  // see live_reload_extended example for explanations
  rerun_if_local_dependencies_change();
  println!("cargo:rerun-if-changed=");
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
  let mut visited_deps = HashSet::new();
  for dep_id in &pkg_graph.dependencies {
    add_rerun_if_changed_for_deps(dep_id, &metadata, &mut visited_deps);
  }
}

fn add_rerun_if_changed_for_deps(
  dep_id: &PackageId,
  metadata: &Metadata,
  visited_deps: &mut HashSet<PackageId>,
) {
  if !visited_deps.insert(dep_id.clone()) {
    // Already visited
    return;
  }

  let Some(dep_pkg) = metadata.packages.iter().find(|p| p.id == *dep_id) else {
    return;
  };

  // We only need local dependencies
  if dep_pkg.source.is_some() {
    return;
  }

  // The manifest_path is the path to Cargo.toml, so we get its parent directory.
  let dep_dir = Path::new(&dep_pkg.manifest_path).parent().unwrap();
  println!("cargo:rerun-if-changed={}", dep_dir.to_str().unwrap());

  // Recursively check dependencies of this local dependency
  if let Some(resolved_node) = metadata
    .resolve
    .as_ref()
    .unwrap()
    .nodes
    .iter()
    .find(|n| n.id == *dep_id)
  {
    for transitive_dep_id in &resolved_node.dependencies {
      add_rerun_if_changed_for_deps(transitive_dep_id, metadata, visited_deps);
    }
  }
}
