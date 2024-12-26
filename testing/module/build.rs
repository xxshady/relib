fn main() {
  if cfg!(feature = "unloading") {
    relib_interface::module::generate(
      test_shared::unloading::EXPORTS,
      "test_shared::unloading::exports::Exports",
      test_shared::unloading::IMPORTS,
      "test_shared::unloading::imports::Imports",
    );
  } else {
    relib_interface::module::generate(
      test_shared::no_unloading::EXPORTS,
      "test_shared::no_unloading::exports::Exports",
      test_shared::no_unloading::IMPORTS,
      "test_shared::no_unloading::imports::Imports",
    );
  }
}
