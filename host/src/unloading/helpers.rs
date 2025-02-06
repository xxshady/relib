pub fn unrecoverable(message: &str) -> ! {
  let message = format!("something unrecoverable happened: {message}");
  unrecoverable_impl(&message);
}

pub fn unrecoverable_with_prefix(message: &str, prefix: &str) -> ! {
  let message = format!("[{prefix}] something unrecoverable happened: {message}");
  unrecoverable_impl(&message);
}

fn unrecoverable_impl(message: &str) -> ! {
  eprintln!("{message}");

  let backtrace = std::backtrace::Backtrace::capture();
  eprintln!("backtrace: {backtrace}");

  eprintln!("aborting");
  std::process::abort();
}
