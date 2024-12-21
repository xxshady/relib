#![allow(unused_imports)]

use std::thread::sleep;
use std::{io::stdin, thread, time::Duration};

use relib_host::exports_types::ModuleValue;
use relib_host::{Module, ModuleExportsForHost};

relib_interface::include_exports!();
relib_interface::include_imports!();
use gen_exports::ModuleExports;
use gen_imports::{init_imports, ModuleImportsImpl};

use testing_shared::imports::Imports;

impl Imports for ModuleImportsImpl {
  fn b() -> i32 {
    panic!()
  }
}

pub fn main() {
  #[cfg(feature = "unloading")]
  test_unloading();
  #[cfg(not(feature = "unloading"))]
  let _ = load::<()>();
}

#[cfg(feature = "unloading")]
fn test_unloading() {
  for _ in 1..=12 {
    load::<()>().unload().unwrap();
  }

  for _ in 1..=100 {
    println!();
  }

  module_load_loop();
}

fn module_load_loop() {
  fn print_memory_use() {
    let stats = memory_stats::memory_stats().unwrap();
    let bytes = stats.virtual_mem;

    let memory = (bytes as f64) / 1024. / 1024.;
    println!("[host] memory in use: {memory:.2}mb");
  }
  print_memory_use();

  let mut unload_immediately = true;
  let mut clear_console = false;
  loop {
    if clear_console {
      clear_console = false;

      std::process::Command::new("clear").status().unwrap();

      let mut message = String::new();
      stdin().read_line(&mut message).unwrap();
    }

    println!("[host] loading module");
    let module = load::<ModuleExports>();

    let a: Option<ModuleValue<i32>> = unsafe { module.exports().a() };
    dbg!(a.as_ref().map(|v| **v) == Some(i32::MAX), a);

    let b: Option<ModuleValue<u8>> = unsafe { module.exports().b() };
    dbg!(b);

    print_memory_use();

    if unload_immediately {
      println!("[host] unloading module");
      module.unload().unwrap_or_else(|e| {
        panic!("{e:#}");
      });
      print_memory_use();
    } else {
      let mut _message = String::new();
      stdin().read_line(&mut _message).unwrap();

      println!("[host] unloading module");
      module.unload().unwrap_or_else(|e| {
        panic!("{e:#}");
      });
      print_memory_use();
    }

    let mut message = String::new();
    stdin().read_line(&mut message).unwrap();

    if message == "q\n" {
      return;
    } else if message == "stop\n" {
      unload_immediately = !unload_immediately;
      if unload_immediately {
        println!("[host] module will be unloaded immediately");
      } else {
        println!("[host] module won't be unloaded immediately");
      }
    } else if message == "clear\n" {
      clear_console = true;
    }
  }

  // only two times
  // for _ in 1..=2 {
  //   println!("[host] loading module");
  //   let module = load();

  //   print_memory_use();

  //   if unload_immediately {
  //     println!("[host] unloading module");
  //     module.unload().unwrap_or_else(|e| {
  //       panic!("{e:#}");
  //     });
  //     print_memory_use();
  //   }
  // }
}

fn load<T: ModuleExportsForHost>() -> Module<T> {
  let directory = if cfg!(debug_assertions) {
    "debug"
  } else {
    "release"
  };

  let path = if cfg!(target_os = "linux") {
    format!("target/{directory}/libtest_module.so")
  } else {
    format!("target/{directory}/test_module.dll")
  };

  let module = relib_host::load_module::<T>(path, init_imports).unwrap_or_else(|e| {
    panic!("{e:#}");
  });

  // dbg!();
  unsafe {
    module.call_main::<()>().unwrap();
    // dbg!(returned);
  }
  // dbg!();

  // let a = module.exports().a().unwrap();
  // dbg!(&a);
  // let a = *a;

  // let b = unsafe { module.exports().b() }.unwrap();
  // dbg!(b);

  // module.unload().unwrap_or_else(|e| {
  //   panic!("{e:#}");
  // });

  // todo!()
  module
}
