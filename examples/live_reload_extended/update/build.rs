fn main() {
  relib_interface::module::generate_with_prefix(
    "update",
    update_contract::EXPORTS,
    "update_contract::exports::Exports",
    main_contract::IMPORTS,
    "main_contract::imports::Imports",
  );
}
