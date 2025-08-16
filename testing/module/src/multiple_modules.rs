#[relib_module::export]
pub fn main() {
  println!(
    "[module] thread id: {:?} (using thread_id crate)",
    thread_id::get()
  );
  println!(
    "[module] std current thread id: {:?}",
    std::thread::current().id()
  );

  for i in 0..3 {
    std::thread::sleep_ms(10);
    println!("{}___END", format!("{i}").repeat(10));
    // std::thread::sleep_ms(10);
    // dbg!(format!("{}___END", format!("{i}").repeat(10)));
  }
}
