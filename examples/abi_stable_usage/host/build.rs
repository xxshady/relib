fn main() {
  relib_interface::host::generate(
    shared::EXPORTS,
    "shared::exports::Exports",
    shared::IMPORTS,
    "shared::imports::Imports",
  );
}
