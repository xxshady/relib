use std::thread::sleep;
use std::{io::stdin, thread, time::Duration};

use relib_host::exports_types::ModuleValue;
use relib_host::{Module, ModuleExportsForHost};

use crate::shared::load;

relib_interface::include_exports!();
relib_interface::include_imports!();
use gen_exports::ModuleExports;
use gen_imports::{init_imports, ModuleImportsImpl};

use test_shared::unloading::imports::Imports;

impl Imports for ModuleImportsImpl {
  fn b() {
    panic!()
  }

  fn with_return_value() -> Vec<u8> {
    vec![1_u8; 1024 * 1024 * 100]
  }
}

pub fn main() {
  for _ in 1..=12 {
    load::<()>(init_imports).unload().unwrap();
  }

  test_unloading_features();
}

#[cfg(not(feature = "unloading"))]
fn test_without_unloading() {
  let _ = load::<()>();
}

#[cfg(feature = "unloading")]
fn test_unloading_features() {
  print_memory_use();

  for _ in 1..=2 {
    println!("[host] loading module");
    let module = load::<gen_exports::ModuleExports>(init_imports);

    print_memory_use();

    println!("[host] unloading module");
    module.unload().unwrap_or_else(|e| {
      panic!("{e:#}");
    });

    print_memory_use();
  }
}

fn print_memory_use() {
  let stats = memory_stats::memory_stats().unwrap();
  let bytes = stats.virtual_mem;
  let megabytes = (bytes as f64) / 1024. / 1024.;

  println!("[host] memory in use: {megabytes:.2}mb");
}
