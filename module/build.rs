fn main() {
  relib_interface::module::generate_internal(
    relib_internal_shared::EXPORTS,
    "relib_internal_shared::exports::___Internal___Exports___",
    relib_internal_shared::IMPORTS,
    "relib_internal_shared::imports::___Internal___Imports___",
  );
  relib_internal_crate_compilation_info::provide();
}
