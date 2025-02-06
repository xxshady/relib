use std::backtrace::Backtrace;

#[relib_module::export]
pub fn main() {
  if cfg!(debug_assertions) {
    let backtrace = Backtrace::force_capture();
    let backtrace = format!("{backtrace}");
    assert!(backtrace.contains("testing\\module\\src\\backtrace_unloading.rs:6"));
  } else {
    #[inline(never)]
    #[unsafe(no_mangle)]
    fn testing_release_backtrace____() -> Backtrace {
      Backtrace::force_capture()
    }

    let backtrace = testing_release_backtrace____();
    let backtrace = format!("{backtrace}");
    assert!(
      backtrace.contains("testing_release_backtrace____"),
      "backtrace was:\n{backtrace}"
    );
  }
}
