fn main() {
  relib_interface::module::generate_imports(
    main_contract::SHARED_IMPORTS,
    "main_contract::SharedImports",
  );
}
