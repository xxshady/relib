#![allow(dead_code)]

relib_interface::include_imports!();

// includes `mod gen_exports`
relib_interface::include_exports!();
use gen_exports::ModuleExportsImpl;

use testing_shared::exports::Exports;

impl Exports for ModuleExportsImpl {
  fn a() -> i32 {
    10
  }

  fn b() -> u8 {
    22
  }
}

#[relib_module::export]
fn main() -> i32 {
  println!("[module] hello world");

  struct TlsWithDrop(Vec<u8>);

  impl Drop for TlsWithDrop {
    fn drop(&mut self) {
      println!("[module] drop was called");
      // std::mem::forget(std::mem::take(&mut self.0));
    }
  }

  thread_local! {
    static TLS_WITH_DROP: TlsWithDrop = TlsWithDrop(
      // allocate 200mb
      vec![1_u8; 1024 * 1024 * 200]
    );
  }

  // initialize it
  TLS_WITH_DROP.with(|_| {});

  // // background threads are checked when this program
  // // is unloaded
  // use std::thread;
  // thread::spawn(|| {
  //   use std::time::Duration;
  //   thread::sleep(Duration::from_secs(1000000));
  // });

  let _value = unsafe { gen_imports::b() };

  123
}

#[relib_module::export]
fn before_unload() {
  dbg!();
}
