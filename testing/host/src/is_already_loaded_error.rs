use {
  crate::shared::{init_module_imports, load_module},
  cfg_if::cfg_if,
  relib_host::{LoadError, Module, ModuleExportsForHost},
  test_host_shared::load_module_with_result,
};

pub fn main() {
  let (module, _) = load_module::<(), ()>(init_module_imports, true);

  let result = load_module_with_result::<(), ()>(init_module_imports, true);
  let Err(LoadError::ModuleAlreadyLoaded) = result else {
    panic!("expected ModuleAlreadyLoaded");
  };

  unload_module(module);

  let result = load_module_with_result::<(), ()>(init_module_imports, true);
  let Ok(_) = result else {
    panic!("expected Ok");
  };
}

fn unload_module<E: ModuleExportsForHost>(module: Module<E>) {
  cfg_if! {
    if #[cfg(feature = "is_already_loaded_error")] {
      module.unload().unwrap();
    } else {
      drop(module);
      panic!("this branch must not be called");
    }
  }
}
