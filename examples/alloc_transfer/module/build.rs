fn main() {
  relib_interface::module::generate(
    shared::EXPORTS,
    "shared::Exports",
    shared::IMPORTS,
    "shared::Imports",
  );
}
