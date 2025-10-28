use {
  crate::shared::alloc_some_bytes,
  std::{backtrace::Backtrace, mem::forget, path::MAIN_SEPARATOR},
};

#[relib_module::export]
pub fn main() {
  println!("[module] main");

  if cfg!(feature = "code_change_leak") {
    println!("[module] leak");
    forget(alloc_some_bytes());
  }

  let s = MAIN_SEPARATOR;

  if cfg!(feature = "code_change_backtrace_unloading") {
    println!("code_change_backtrace_unloading");

    if cfg!(debug_assertions) {
      let backtrace = Backtrace::force_capture();
      let backtrace = format!("{backtrace}");
      assert!(
        backtrace.contains(&format!("testing{s}module{s}src{s}code_change.rs:20")),
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
      assert!(backtrace.contains(&format!("testing{s}module{s}src{s}code_change.rs:48")));
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
