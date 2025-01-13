use std::mem::forget;

use crate::shared::alloc_some_bytes;

#[relib_module::export]
pub fn main() {
  println!("[module] main");

  if cfg!(feature = "code_change_leak") {
    println!("[module] leak");
    forget(alloc_some_bytes());
  }
}

#[cfg(feature = "code_change_before_unload")]
#[relib_module::export]
pub fn before_unload() {
  println!("[module] before_unload");
}
