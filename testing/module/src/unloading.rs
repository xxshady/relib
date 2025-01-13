//   // let res = std::thread::spawn(|| unsafe {
//   //   gen_imports::b();
//   //   // panic!();
//   // })
//   // .join();
//   // let _ = dbg!(res);

#[relib_module::export]
pub fn main() {
  println!("[module] hello world");

  // seems like standard library caches something for threads in memory
  // (it doesn't increase memory usage between module reloads though)
  // so we need to spawn at least one thread here to "warm up" the memory for testing
  std::thread::spawn(|| {}).join().unwrap();
}

#[relib_module::export]
fn before_unload() {
  dbg!();
}
