use std::{backtrace::Backtrace, path::MAIN_SEPARATOR};

#[relib_module::export]
pub fn main() {
  if cfg!(debug_assertions) {
    let backtrace = Backtrace::force_capture();
    let backtrace = format!("{backtrace}");
    let s = MAIN_SEPARATOR;
    assert!(
      backtrace.contains(&format!(
        "testing{s}module{s}src{s}backtrace_unloading_host_as_dylib.rs:6"
      )),
      "backtrace was:\n{backtrace}",
    );
    assert!(
      backtrace.contains(&format!("testing{s}host_as_dylib{s}src{s}lib.rs:30")),
      "backtrace was:\n{backtrace}",
    );
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
    assert!(
      backtrace.contains("testing_release_backtrace_in_host____"),
      "backtrace was:\n{backtrace}"
    );
  }
}
