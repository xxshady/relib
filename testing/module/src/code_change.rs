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

    if cfg!(debug_assertions) {
      let backtrace = Backtrace::force_capture();
      let backtrace = format!("{backtrace}");
      assert!(
        backtrace.contains("testing\\module\\src\\code_change.rs:18"),
        "backtrace was:\n{backtrace}"
      );
    } else {
      #[inline(never)]
      #[unsafe(no_mangle)]
      fn testing_release_backtrace_code_change____() -> (Backtrace, String) {
        (
          Backtrace::force_capture(),
          // a hack to prevent optimization
          String::from("awdfkjgkfjgfg"),
        )
      }

      let (backtrace, _) = testing_release_backtrace_code_change____();
      let backtrace = format!("{backtrace}");
      assert!(
        backtrace.contains("testing_release_backtrace_code_change____"),
        "backtrace was:\n{backtrace}"
      );
    }
  }

  if cfg!(feature = "code_change_backtrace_unloading2") {
    if cfg!(debug_assertions) {
      let backtrace = Backtrace::force_capture();
      let backtrace = format!("{backtrace}");
      assert!(backtrace.contains("testing\\module\\src\\code_change.rs:46"));
    } else {
      #[inline(never)]
      #[unsafe(no_mangle)]
      fn testing_release_backtrace_code_change2____() -> Backtrace {
        Backtrace::force_capture()
      }

      let backtrace = testing_release_backtrace_code_change2____();
      let backtrace = format!("{backtrace}");
      assert!(
        backtrace.contains("testing_release_backtrace_code_change2____"),
        "backtrace was:\n{backtrace}"
      );
    }
  }
}

#[cfg(feature = "code_change_before_unload")]
#[relib_module::export]
pub fn before_unload() {
  println!("[module] before_unload");
}
