use abi_stable::std_types::RVec;
use shared::imports::Imports;

relib_interface::include_exports!();
use gen_exports::ModuleExports;

relib_interface::include_imports!();
use gen_imports::{init_imports, ModuleImportsImpl};

impl Imports for ModuleImportsImpl {
  fn foo() -> RVec<u8> {
    vec![1, 2, 3].into()
  }
}

fn main() {
  let path_to_dylib = if cfg!(target_os = "linux") {
    "target/debug/libmodule.so"
  } else {
    "target/debug/module.dll"
  };

  let module = relib_host::load_module::<ModuleExports>(path_to_dylib, init_imports)
    .unwrap_or_else(|e| {
      panic!("module loading failed: {e:#}");
    });

  unsafe { module.call_main::<()>() }.unwrap();

  let bar_value = unsafe { module.exports().bar() }.unwrap();
  let string = bar_value.clone();
  dbg!(string);

  // module.unload() is provided when unloading feature of relib_host crate is enabled
  #[cfg(feature = "unloading")]
  {
    println!("unloading feature is enabled, calling module unload");

    module.unload().unwrap_or_else(|e| {
      panic!("module unloading failed: {e:#}");
    });
  }
}
