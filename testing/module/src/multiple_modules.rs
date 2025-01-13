#[relib_module::export]
pub fn main() {
  println!("[module] thread id: {:?}", thread_id::get());

  // TODO: std::thread::current() not working correctly and causes seg fault
  // https://github.com/xxshady/relib/issues/4
  // println!("[module] std thread id: {:?}", std::thread::current().id());
}
