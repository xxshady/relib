fn main() {
  relib_interface::host::generate(
    shared::EXPORTS,
    "shared::exports::Exports",
    shared::IMPORTS,
    "shared::imports::Imports",
  );

  relib_interface::host::generate_with_prefix(
    "module2",
    shared::EXPORTS2,
    "shared::exports2::Exports2",
    shared::IMPORTS2,
    "shared::imports2::Imports2",
  );
}
