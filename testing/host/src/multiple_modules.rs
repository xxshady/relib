use std::thread;

use cfg_if::cfg_if;

use relib_host::{Module, ModuleExportsForHost};
use crate::shared::{self, init_module_imports};

pub fn main() {
  thread::scope(|s| {
    for idx in 0..10 {
      s.spawn(move || {
        let module = load_module(&format!("test_module_{idx}"));
        unload_module(module);
      });
    }
  });

  // TODO: fix std::thread::current() https://github.com/xxshady/relib/issues/4
  // thread::spawn(|| {
  //   let module = load_module(&format!("test_module_0"));
  //   unload_module(module);

  //   thread::sleep(Duration::from_millis(1000));
  // })
  // .join()
  // .unwrap();
}

fn unload_module<E: ModuleExportsForHost>(module: Module<E>) {
  cfg_if! {
    if #[cfg(feature = "multiple_modules")] {
      let id = module.id;
      module.unload().unwrap();
      println!("{:?} unloaded: {id}", std::thread::current().id());
    } else {
      drop(module);
      panic!("this branch must not be called");
    }
  }
}

fn load_module(name: &str) -> Module<()> {
  let (module, _) = shared::load_module_with_name::<(), ()>(init_module_imports, name, true);
  module
}
