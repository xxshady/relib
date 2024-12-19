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
  // replace "?" with your file name, for example if you named module crate as "module"
  // on linux the path will be "target/debug/libmodule.so", on windows it will be "target/debug/module.dll"
  let path_to_dylib = "target/debug/?";

  let module = relib_host::load_module::<ModuleExports>(path_to_dylib, init_imports).unwrap();

  unsafe { module.call_main::<()>() }.unwrap();

  let bar_value = unsafe { module.exports().bar() }.unwrap();
  let string = bar_value.clone();
  dbg!(string);

  module.unload().unwrap_or_else(|e| {
    panic!("module unloading failed: {e:#}");
  });
}
