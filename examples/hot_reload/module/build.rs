fn main() {
  relib_interface::module::generate(
    main_contract::EXPORTS,
    "main_contract::exports::Exports",
    main_contract::IMPORTS,
    "main_contract::imports::Imports",
  );
}
