use {
  crate::shared::{init_module_imports, load_module},
  cfg_if::cfg_if,
  relib_host::{Module, ModuleExportsForHost},
  std::io::{Write, stderr, stdin},
  test_shared::print_memory_use,
};

pub fn main() {
  println!("start");

  let mut s = String::new();
  let mut i = 0;
  loop {
    print_memory_use();

    let (module, returned) = load_module::<(), ()>(init_module_imports, true);
    returned.unwrap();

    unload_module(module);

    print_memory_use();

    i += 1;
    let mut stderr = stderr().lock();
    let message = format!("code_change_module_has_been_exec_{i}\n");
    stderr.write_all(message.as_bytes()).unwrap();
    stderr.flush().unwrap();

    println!("waiting");
    stdin().read_line(&mut s).unwrap();

    println!("received: {s:?}");

    if s.trim() == "end" {
      println!("received end");
      eprintln!("received_end_______________");
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
