//   // let res = std::thread::spawn(|| unsafe {
//   //   gen_imports::b();
//   //   // panic!();
//   // })
//   // .join();
//   // let _ = dbg!(res);

use crate::shared::alloc_some_bytes;

#[relib_module::export]
pub fn main() {
  println!("[module] main called");

  let vec = alloc_some_bytes();
  println!("[module] allocated {} bytes", vec.len());
  drop(vec);

  // seems like standard library caches something for threads in memory on linux
  // (it doesn't increase memory usage between module reloads though)
  // so we need to spawn at least one thread here to "warm up" the memory for testing
  std::thread::spawn(|| {}).join().unwrap();
}

#[relib_module::export]
fn before_unload() {
  dbg!();
}
