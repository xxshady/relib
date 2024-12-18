fn main() {
  relib_interface::module::generate(
    testing_shared::EXPORTS,
    "testing_shared::exports::Exports",
    testing_shared::IMPORTS,
    "testing_shared::imports::Imports",
  );
}
