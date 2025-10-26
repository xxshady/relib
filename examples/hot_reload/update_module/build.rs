fn main() {
  relib_interface::module::generate_with_prefix(
    "update",
    update_contract::EXPORTS,
    "update_contract::Exports",
    main_contract::EMPTY_IMPORTS,
    "main_contract::EmptyImports",
  );
}
