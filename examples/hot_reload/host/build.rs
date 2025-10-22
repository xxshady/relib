fn main() {
  relib_interface::host::generate(
    main_contract::EXPORTS,
    "main_contract::exports::Exports",
    main_contract::SHARED_IMPORTS,
    "main_contract::shared_imports::SharedImports",
  );

  relib_interface::host::generate_with_prefix(
    "update",
    update_contract::EXPORTS,
    "update_contract::exports::Exports",
    main_contract::SHARED_IMPORTS,
    "main_contract::shared_imports::SharedImports",
  );
}
