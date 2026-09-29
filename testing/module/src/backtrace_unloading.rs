use std::{backtrace::Backtrace, hint::black_box, path::MAIN_SEPARATOR};

#[relib_module::export]
pub fn main() {
  if cfg!(debug_assertions) {
    let backtrace = Backtrace::force_capture();
    let backtrace = format!("{backtrace}");
    let s = MAIN_SEPARATOR;
    assert!(backtrace.contains(&format!(
      "testing{s}module{s}src{s}backtrace_unloading.rs:6"
    )));
  } else {
    #[inline(never)]
    #[unsafe(no_mangle)]
    fn testing_release_backtrace____() -> Backtrace {
      black_box(Backtrace::force_capture())
    }

    let backtrace = testing_release_backtrace____();
    let backtrace = format!("{backtrace}");
    assert!(
      backtrace.contains("testing_release_backtrace____"),
      "backtrace was:\n{backtrace}"
    );
  }
}
