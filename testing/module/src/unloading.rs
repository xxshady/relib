relib_interface::include_imports!();
relib_interface::include_exports!();
use gen_exports::ModuleExportsImpl;

use test_shared::unloading::exports::Exports;

impl Exports for ModuleExportsImpl {
  fn a() -> i32 {
    10
  }

  fn b() -> u8 {
    22
  }
}

pub fn main() {
  println!("[module] hello world");

  // struct TlsWithDrop(Vec<u8>);

  // impl Drop for TlsWithDrop {
  //   fn drop(&mut self) {
  //     println!("[module] drop was called");
  //     // std::mem::forget(std::mem::take(&mut self.0));
  //   }
  // }

  // thread_local! {
  //   static TLS_WITH_DROP: TlsWithDrop = TlsWithDrop(
  //     // allocate 200mb
  //     vec![1_u8; 1024 * 1024 * 200]
  //   );
  // }

  // // initialize it
  // TLS_WITH_DROP.with(|_| {});

  // // background threads are checked when this program
  // // is unloaded
  // use std::thread;
  // thread::spawn(|| {
  //   use std::time::Duration;
  //   thread::sleep(Duration::from_secs(1000000));
  // });

  // let _value = unsafe { gen_imports::b() };

  // dbg!();
  // std::thread::spawn(|| {
  //   TLS_WITH_DROP.with(|_| {});
  // })
  // .join()
  // .unwrap();
  // dbg!();

  // std::env::set_var("RUST_BACKTRACE", "1");
  // panic!();

  // let res = std::thread::spawn(|| unsafe {
  //   gen_imports::b();
  //   // panic!();
  // })
  // .join();
  // let _ = dbg!(res);

  let return_value = unsafe { gen_imports::with_return_value() };
  dbg!(return_value.len());
  std::mem::forget(return_value);
}

#[relib_module::export]
fn before_unload() {
  dbg!();
}
