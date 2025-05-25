use cfg_if::cfg_if;

use relib_host::{Module, ModuleExportsForHost};
use crate::shared::{self, init_module_imports, ModuleExports};

pub fn main() {
  let module = load_module("windows_background_threads__test_module_0");

  unsafe {
    module
      .exports()
      .spawn_background_threads(module.id, 3)
      .unwrap();
  }

  unload_module(module);
}

fn unload_module<E: ModuleExportsForHost>(module: Module<E>) {
  cfg_if! {
    if #[cfg(feature = "windows_background_threads_fail")] {
      use relib_host::UnloadError;

      let Err(UnloadError::ThreadsStillRunning(_)) = dbg!(module.unload()) else {
        unreachable!();
      };
    } else {
      drop(module);
      panic!("this branch must not be called");
    }
  }
}

fn load_module(name: &str) -> Module<ModuleExports> {
  let (module, _) =
    shared::load_module_with_name::<ModuleExports, ()>(init_module_imports, name, true);
  module
}
