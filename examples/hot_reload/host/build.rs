fn main() {
  relib_interface::host::generate_with_prefix(
    "main_module",
    main_contract::EXPORTS,
    "main_contract::Exports",
    main_contract::SHARED_IMPORTS,
    "main_contract::SharedImports",
  );

  relib_interface::host::generate_with_prefix(
    "update_module",
    update_contract::EXPORTS,
    "update_contract::Exports",
    main_contract::SHARED_IMPORTS,
    "main_contract::SharedImports",
  );
}
