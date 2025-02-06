use std::{env, panic::set_hook, thread};

use super::gen_imports;

pub fn init() {
  set_hook(Box::new(|info| {
    let current_thread = thread::current();
    let thread_name = current_thread.name().unwrap_or("<unnamed>");
    let backtrace_message = backtrace_message();
    let panic_message = format!("thread '{thread_name}' {info}\n{backtrace_message}");

    unsafe {
      gen_imports::eprintln(panic_message.as_str().into());
    }

    backtrace::clear_symbol_cache();
  }));
}

fn backtrace_message() -> String {
  #[expect(unused_assignments)]
  let mut backtrace_var_value = Default::default();
  let backtrace_var = match env::var("RUST_BACKTRACE") {
    Ok(v) => {
      backtrace_var_value = v;
      Ok(backtrace_var_value.as_str())
    }
    Err(e) => Err(e),
  };

  match backtrace_var {
    Ok("0") => {
      "note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace".to_owned()
    }

    // TODO: short backtrace when `RUST_BACKTRACE=1`
    // https://github.com/xxshady/relib/issues/5
    Ok(_) => {
      let backtrace = backtrace::Backtrace::new();
      format!("stack backtrace:\n{backtrace:?}")
    }
    _ => "note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace".to_owned(),
  }
}
