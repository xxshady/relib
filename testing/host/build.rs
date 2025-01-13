fn main() {
  relib_interface::host::generate(
    test_shared::EXPORTS,
    "test_shared::exports::Exports",
    test_shared::IMPORTS,
    "test_shared::imports::Imports",
  );
}
