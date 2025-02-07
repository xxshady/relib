use std::backtrace::Backtrace;

use crate::shared::{init_module_imports, load_module};

pub fn main() {
  unsafe {
    relib_host::init();
  }

  let backtrace = Backtrace::force_capture();
  let _ = format!("{backtrace}");

  let (_, _) = load_module::<(), ()>(init_module_imports, true);
}
