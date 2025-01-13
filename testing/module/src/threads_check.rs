use std::{thread, time::Duration};

#[relib_module::export]
pub fn main() {
  thread::spawn(|| {
    thread::sleep(Duration::from_secs(1000000));
  });
}
