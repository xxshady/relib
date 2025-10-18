fn main() {
  relib_interface::host::generate(
    main_contract::EXPORTS,
    "main_contract::exports::Exports",
    main_contract::IMPORTS,
    "main_contract::imports::Imports",
  );

  relib_interface::host::generate_with_prefix(
    "update",
    update_contract::EXPORTS,
    "update_contract::exports::Exports",
    main_contract::IMPORTS,
    "main_contract::imports::Imports",
  );
}
