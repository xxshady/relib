use relib_host::Module;

use test_shared::no_unloading::imports::Imports;
use crate::shared::load;

relib_interface::include_exports!();
relib_interface::include_imports!();
use gen_exports::ModuleExports;
use gen_imports::{init_imports, ModuleImportsImpl};

impl Imports for ModuleImportsImpl {
  fn b() {
    panic!()
  }

  fn with_return_value(value: i32) -> i32 {
    value
  }
}

pub fn main() {
  let _ = load::<ModuleExports>(init_imports);
}
