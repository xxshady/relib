use std::thread::sleep;
use std::{io::stdin, thread, time::Duration};

use abi_stable::std_types::{RStr, RString, RVec};
use relib_host::{Module, ModuleExportsForHost};

use crate::shared::load;

relib_interface::include_exports!();
relib_interface::include_imports!();
use gen_exports::ModuleExports;
use gen_imports::{init_imports, ModuleImportsImpl};

use test_shared::unloading::imports::Imports;

impl Imports for ModuleImportsImpl {
  fn a() -> i32 {
    10
  }

  fn b(r: RStr) -> RString {
    dbg!();
    r.to_owned().repeat(100_000).into()
  }

  fn b2(r: RStr, r2: RStr) -> RString {
    dbg!();
    assert_eq!(r, r2);
    r2.to_owned().repeat(100_000).into()
  }

  fn d() {}

  fn ptr() -> *const i32 {
    Box::into_raw(Box::new(123))
  }
}

pub fn main() {
  // for _ in 1..=12 {
  //   load::<()>(init_imports).unload().unwrap();
  // }

  // test_unloading_features();

  let module = load::<ModuleExports>(init_imports);

  // dbg!(unsafe { module.exports().a() });

  dbg!();

  print_memory_use();

  let value = unsafe { module.exports().b("a".repeat(1024 * 1024).as_str().into()) };
  print_memory_use();

  dbg!(value.map(|v| v.len()));

  print_memory_use();

  module.unload().unwrap();
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
