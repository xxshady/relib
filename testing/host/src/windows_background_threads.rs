use {
  crate::shared::{self, ModuleExports, init_module_imports},
  cfg_if::cfg_if,
  relib_host::{Module, ModuleExportsForHost},
  std::{thread, time::Duration},
};

pub fn main() {
  thread::scope(|s| {
    let first_module_handle = s.spawn(move || {
      let module = load_module("windows_background_threads__test_module_0");

      unsafe {
        module
          .exports()
          .spawn_background_threads(module.id(), 3)
          .unwrap();
      }

      thread::park();

      unsafe {
        module.exports().join_background_threads().unwrap();
      }

      unload_module(module);
    });

    // wait for threads of first module to spawn
    thread::sleep(Duration::from_secs(1));

    s.spawn(move || {
      let module = load_module("windows_background_threads__test_module_1");
      unsafe {
        module
          .exports()
          .spawn_background_threads(module.id(), 5)
          .unwrap();
      }

      unsafe {
        module.exports().join_background_threads().unwrap();
      }

      unload_module(module);
    })
    .join()
    .unwrap();

    first_module_handle.thread().unpark();
  });
}

fn unload_module<E: ModuleExportsForHost>(module: Module<E>) {
  cfg_if! {
    if #[cfg(feature = "windows_background_threads")] {
      let id = module.id();
      module.unload().unwrap();
      println!("{:?} unloaded module: {id}", std::thread::current().id());
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
