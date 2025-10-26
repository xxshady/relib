fn main() {
  relib_interface::module::generate_exports_with_prefix(
    "update",
    update_contract::EXPORTS,
    "update_contract::Exports",
  );
}
