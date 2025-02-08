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
}
