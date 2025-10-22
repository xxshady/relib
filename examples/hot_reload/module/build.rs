fn main() {
  relib_interface::module::generate(
    main_contract::EXPORTS,
    "main_contract::exports::Exports",
    main_contract::SHARED_IMPORTS,
    "main_contract::shared_imports::SharedImports",
  );
}
