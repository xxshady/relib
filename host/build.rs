fn main() {
  relib_interface::host::generate_internal(
    relib_internal_shared::EXPORTS,
    "relib_internal_shared::exports::___Internal___Exports___",
    relib_internal_shared::IMPORTS,
    "relib_internal_shared::imports::___Internal___Imports___",
  );

  relib_interface::host::generate_internal_exports_with_prefix(
    "internal_generated_module_no_unloading",
    relib_internal_shared::EXPORTS_NO_UNLOADING,
    "relib_internal_shared::exports_no_unloading::___Exports___NoUnloading___",
  );

  relib_internal_crate_compilation_info::provide();
}
