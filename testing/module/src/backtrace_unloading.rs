use std::backtrace::Backtrace;

#[relib_module::export]
pub fn main() {
  let backtrace = Backtrace::force_capture();
  let _ = format!("{backtrace}");
}
