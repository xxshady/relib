use std::io::stdin;

use cfg_if::cfg_if;
use relib_host::{Module, ModuleExportsForHost};

use crate::shared::{init_module_imports, load_module};

pub fn main() {
  println!("start");

  let mut s = String::new();
  loop {
    let (module, _) = load_module::<(), ()>(init_module_imports);
    unload_module(module);

    println!("waiting");
    stdin().read_line(&mut s).unwrap();

    println!("received: {s:?}");

    if s.trim() == "end" {
      println!("received end");
      break;
    }
    s.clear();
  }
}

fn unload_module<E: ModuleExportsForHost>(module: Module<E>) {
  cfg_if! {
    if #[cfg(feature = "code_change")] {
      module.unload().unwrap();
      println!("unloaded");
    } else {
      drop(module);
      panic!("this branch must not be called");
    }
  }
}
