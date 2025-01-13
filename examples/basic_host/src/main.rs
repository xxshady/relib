fn main() {
  let path_to_dylib = std::env::args().nth(1).unwrap();

  // `()` means empty imports and exports, here module doesn't import or export anything
  let module = relib_host::load_module::<()>(path_to_dylib, ()).unwrap_or_else(|e| {
    panic!("module loading failed: {e:#}");
  });

  // main function is unsafe to call (as well as any other module export) because these preconditions are not checked by relib:
  // 1. returned value must be actually `R` at runtime, for example you called this function with type bool but module returns i32.
  // 2. type of return value must be FFI-safe.
  // 3. returned value must not be a reference-counting pointer (see limitations on main docs page/README).
  let returned_value: Option<()> = unsafe { module.call_main::<()>() };

  // if module panics while executing any export it returns None
  // (panic will be printed by module)
  if returned_value.is_none() {
    println!("module panicked");
  }

  // module.unload() is provided when unloading feature of relib_host crate is enabled
  #[cfg(feature = "unloading")]
  {
    println!("unloading feature is enabled, calling module unload");

    module.unload().unwrap_or_else(|e| {
      panic!("module unloading failed: {e:#}");
    });
  }
}
