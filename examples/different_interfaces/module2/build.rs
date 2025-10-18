fn main() {
  relib_interface::module::generate_with_prefix(
    "module2",
    shared::EXPORTS2,
    "shared::exports2::Exports2",
    shared::IMPORTS2,
    "shared::imports2::Imports2",
  );
}
