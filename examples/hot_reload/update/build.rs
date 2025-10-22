fn main() {
  relib_interface::module::generate_with_prefix(
    "update",
    update_contract::EXPORTS,
    "update_contract::exports::Exports",
    main_contract::SHARED_IMPORTS,
    "main_contract::shared_imports::SharedImports",
  );
}
