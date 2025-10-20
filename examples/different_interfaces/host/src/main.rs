use relib_host::{ModuleExportsForHost, InitImports};
use shared::{imports::Imports, imports2::Imports2};

relib_interface::include_exports!();
relib_interface::include_imports!();

relib_interface::include_exports!(gen_exports2, "module2");
relib_interface::include_imports!(gen_imports2, "module2");

impl Imports for gen_imports::ModuleImportsImpl {
  fn foo() -> u8 {
    10
  }
}

impl Imports2 for gen_imports2::ModuleImportsImpl {
  fn foo2() -> u8 {
    20
  }
}

fn main() {
  load_module(
    "module",
    gen_imports::init_imports,
    |exports: &gen_exports::ModuleExports| unsafe {
      exports.bar().unwrap();
    },
  );
  load_module(
    "module2",
    gen_imports2::init_imports,
    |exports: &gen_exports2::ModuleExports| unsafe {
      exports.bar2().unwrap();
    },
  );
}

fn load_module<E: ModuleExportsForHost>(
  name: &str,
  init_imports: impl InitImports,
  call_export: impl FnOnce(&E),
) {
  let path_to_dylib = if cfg!(target_os = "linux") {
    format!("target/debug/lib{name}.so")
  } else {
    format!("target/debug/{name}.dll")
  };

  let module = unsafe { relib_host::load_module::<E>(path_to_dylib, init_imports) };
  let module = module.unwrap_or_else(|e| {
    panic!("module loading failed: {e:#}");
  });

  // main function is unsafe to call (as well as any other module export) because these preconditions are not checked by relib:
  // 1. returned value must be actually `R` at runtime. For example if you called this function with type bool but module returns i32, UB will occur.
  // 2. type of return value must be ABI-stable.
  // 3. returned value must not be a reference-counting pointer or &'static T (see caveats on main docs page/README).
  let returned_value = unsafe { module.call_main::<()>() };

  // if module panics while executing any export it returns None
  // (panic will be printed by module)
  if returned_value.is_none() {
    println!("module panicked");
  }

  call_export(module.exports());

  // module.unload() is provided when unloading feature of relib_host crate is enabled
  #[cfg(feature = "unloading")]
  {
    println!("unloading feature is enabled, calling module unload");

    module.unload().unwrap_or_else(|e| {
      panic!("module unloading failed: {e:#}");
    });
  }
}
