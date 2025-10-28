//! currently it's only used for "backtrace_unloading_host_as_dylib" test

use {
  cfg_if::cfg_if,
  libloading::library_filename,
  relib_host::{Module, ModuleExportsForHost},
  std::path::Path,
};

#[unsafe(no_mangle)]
pub extern "C" fn main() {
  testing_release_backtrace_in_host____();
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

#[inline(never)]
#[unsafe(no_mangle)]
fn testing_release_backtrace_in_host____() {
  let filename = library_filename("test_module");
  let path = Path::new("backtrace_unloading_host_as_dylib").join(filename);

  dbg!();
  let (module, _) = test_host_shared::load_module_with_path::<(), ()>((), &path, true);
  unload_module(module);
  dbg!();
}
