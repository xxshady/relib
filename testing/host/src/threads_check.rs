use cfg_if::cfg_if;
use relib_host::{Module, ModuleExportsForHost};

use crate::shared::{init_module_imports, load_module};

pub fn main() {
  let (module, _) = load_module::<(), ()>(init_module_imports, true);
  unload_module(module);
}

fn unload_module<E: ModuleExportsForHost>(module: Module<E>) {
  cfg_if! {
    if #[cfg(feature = "threads_check")] {
      use relib_host::UnloadError;

      let err = module.unload().unwrap_err();
      assert!(matches!(err, UnloadError::ThreadsStillRunning(..)));

      println!("checked");
    } else {
      drop(module);
      panic!("this branch must not be called");
    }
  }
}
