#[relib_module::export]
fn main() {
  let time = std::time::SystemTime::now();
  std::thread::sleep(std::time::Duration::from_secs(1));

  let time2 = std::time::SystemTime::now();
  println!("diff: {:?}", time2 - time);
}
