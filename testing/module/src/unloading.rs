//   // let res = std::thread::spawn(|| unsafe {
//   //   gen_imports::b();
//   //   // panic!();
//   // })
//   // .join();
//   // let _ = dbg!(res);

#[relib_module::export]
pub fn main() {
  let err = anyhow::anyhow!("err");
  println!("[module] hello world\n{err:?}");
}

#[relib_module::export]
fn before_unload() {
  dbg!();
}
