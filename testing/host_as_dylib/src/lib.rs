//! currently it's only used for "backtrace_unloading_host_as_dylib" test

use cfg_if::cfg_if;
use relib_host::{Module, ModuleExportsForHost};
use test_host_shared::dylib_filename;

#[no_mangle]
pub extern "C" fn main() {
  let filename = dylib_filename("test_module");
  let path = format!("backtrace_unloading_host_as_dylib/{filename}");

  dbg!();
  let (module, _) = test_host_shared::load_module_with_path::<(), ()>((), &path, true);
  unload_module(module);
  dbg!();
}

fn unload_module<E: ModuleExportsForHost>(module: Module<E>) {
  cfg_if! {
    if #[cfg(feature = "backtrace_unloading_host_as_dylib")] {
      module.unload().unwrap();
    } else {
      drop(module);
      panic!("this branch must not be called");
    }
  }
}
