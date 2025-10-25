fn main() {
  relib_interface::module::generate(
    // TODO: add api for partial contract generation?
    // TODO: we dont need imports here
    main_contract::EXPORTS,
    "main_contract::Exports",
    main_contract::EMPTY_IMPORTS,
    "main_contract::EmptyImports",
  );
}
