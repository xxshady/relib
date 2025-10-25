fn main() {
  relib_interface::module::generate(
    // TODO: add api for partial contract generation?
    // TODO: we dont need exports here
    main_contract::EMPTY_EXPORTS,
    "main_contract::exports::EmptyExports",
    main_contract::SHARED_IMPORTS,
    "main_contract::shared_imports::SharedImports",
  );
}
