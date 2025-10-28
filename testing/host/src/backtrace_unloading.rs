use {
  crate::shared::{init_module_imports, load_module},
  cfg_if::cfg_if,
  relib_host::{Module, ModuleExportsForHost},
  test_shared::print_memory_use,
};

pub fn main() {
  for _ in 1..=10 {
    print_memory_use();
    let (module, _) = load_module::<(), ()>(init_module_imports, true);
    unload_module(module);
    print_memory_use();
    println!("-----------------");
  }
}

fn unload_module<E: ModuleExportsForHost>(module: Module<E>) {
  cfg_if! {
    if #[cfg(feature = "backtrace_unloading")] {
      module.unload().unwrap();
    } else {
      drop(module);
      panic!("this branch must not be called");
    }
  }
}
