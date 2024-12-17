fn main() {
  // replace "?" with your file name, for example if you named module crate as "module"
  // on linux the path will be "target/debug/libmodule.so", on windows it will be "target/debug/module.dll"
  let path_to_dylib = std::env::args().nth(1).unwrap();

  // `()` means empty imports and exports, here module doesn't import or export anything
  let module = relib_host::load_module::<()>(path_to_dylib, ()).unwrap();

  // main function is unsafe to call (as well as any other module export) because these preconditions are not checked by relib:
  // 1. returned value must be actually `R` at runtime, for example you called this function with type bool but module returns i32.
  // 2. type of return value must be FFI-safe.
  // (see "Module exports" section for more info about ModuleValue)
  let returned_value: Option<relib_host::ModuleValue<()>> = unsafe { module.call_main::<()>() };

  // if module panics while executing any export it returns None
  // (panic will be printed by module)
  if returned_value.is_none() {
    println!("module panicked");
  }

  module.unload().unwrap_or_else(|e| {
    panic!("module unloading failed: {e:#}");
  });
}
