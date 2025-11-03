fn main() {
  relib_interface::host::generate(
    shared::EXPORTS,
    "shared::Exports",
    shared::IMPORTS,
    "shared::Imports",
  );
}
