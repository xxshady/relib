fn main() {
  // this code assumes that directory and package name of the shared crate are the same
  relib_interface::module::generate(
    shared::EXPORTS,
    "shared::exports::Exports",
    shared::IMPORTS,
    "shared::imports::Imports",
  );
}
