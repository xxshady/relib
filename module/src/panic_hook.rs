use std::panic::set_hook;

use crate::gen_imports;

pub fn init() {
  // TODO: make it more similar to default panic hook (for example, output thread name)
  set_hook(Box::new(|info| {
    // TODO: check env variables? (RUST_BACKTRACE and friends)
    let backtrace = backtrace::Backtrace::new();
    let panic_message = format!("{info}\nbacktrace:\n{backtrace:?}");

    unsafe {
      gen_imports::eprintln(panic_message.as_str().into());
    }

    backtrace::clear_symbol_cache();
  }));
}
