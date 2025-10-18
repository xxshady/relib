fn main() {
  relib_interface::module::generate(
    shared::EXPORTS,
    "shared::exports::Exports",
    shared::IMPORTS,
    "shared::imports::Imports",
  );
}
