fn main() {
  relib_interface::module::generate(
    "../shared/src/exports.rs",
    "testing_shared::exports::Exports",
    "../shared/src/imports.rs",
    "testing_shared::imports::Imports",
  );
}
