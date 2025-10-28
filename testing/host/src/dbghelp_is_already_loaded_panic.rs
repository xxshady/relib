use {
  crate::shared::{init_module_imports, load_module},
  std::backtrace::Backtrace,
};

pub fn main() {
  let backtrace = Backtrace::force_capture();
  let _ = format!("{backtrace}");

  eprintln!("this panic is expected:");
  let result = std::panic::catch_unwind(|| {
    let (_, _) = load_module::<(), ()>(init_module_imports, true);
  });

  let Err(_) = result else {
    panic!("expected dbghelp panic");
  };
}
