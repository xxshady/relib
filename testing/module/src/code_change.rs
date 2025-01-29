use std::{backtrace::Backtrace, mem::forget};

use crate::shared::alloc_some_bytes;

#[relib_module::export]
pub fn main() {
  println!("[module] main");

  if cfg!(feature = "code_change_leak") {
    println!("[module] leak");
    forget(alloc_some_bytes());
  }

  if cfg!(feature = "code_change_backtrace_unloading") {
    println!("code_change_backtrace_unloading");

    let backtrace = Backtrace::force_capture();
    // TODO: add assert
    let _ = format!("{backtrace}");
  }

  if cfg!(feature = "code_change_backtrace_unloading2") {
    let backtrace = Backtrace::force_capture();
    // TODO: add assert
    let _ = format!("{backtrace}");
  }
}

#[cfg(feature = "code_change_before_unload")]
#[relib_module::export]
pub fn before_unload() {
  println!("[module] before_unload");
}
