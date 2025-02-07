mod shared;

mod unloading;
mod no_unloading;
mod exportify;
mod threads_check;
mod before_unload_panic;
mod code_change;
mod multiple_modules;
mod panic_in_interface_module;
mod panic_in_interface_host;
mod backtrace_unloading;
mod backtrace_unloading_host_as_dylib;
mod is_already_loaded_error;
mod dbghelp_is_already_loaded_panic;
mod dbghelp_is_already_loaded_init;

fn main() {
  if cfg!(feature = "unloading") {
    unloading::main();
  } else if cfg!(feature = "no_unloading") {
    no_unloading::main();
  } else if cfg!(feature = "exportify") {
    exportify::main();
  } else if cfg!(feature = "threads_check") {
    threads_check::main();
  } else if cfg!(feature = "before_unload_panic") {
    before_unload_panic::main();
  } else if cfg!(feature = "code_change") {
    code_change::main();
  } else if cfg!(feature = "multiple_modules") {
    multiple_modules::main();
  } else if cfg!(feature = "panic_in_interface_module") {
    panic_in_interface_module::main();
  } else if cfg!(feature = "panic_in_interface_host") {
    panic_in_interface_host::main();
  }
  // depends on mmap_hooks in relib_module crate for linux
  // and dbghelp.dll patches in relib_host crate on windows
  else if cfg!(feature = "backtrace_unloading") {
    backtrace_unloading::main();
  } else if cfg!(feature = "backtrace_unloading_host_as_dylib") {
    backtrace_unloading_host_as_dylib::main();
  } else if cfg!(feature = "is_already_loaded_error") {
    is_already_loaded_error::main();
  } else if cfg!(feature = "dbghelp_is_already_loaded_panic") {
    dbghelp_is_already_loaded_panic::main();
  } else if cfg!(feature = "dbghelp_is_already_loaded_init") {
    dbghelp_is_already_loaded_init::main();
  } else {
    panic!();
  }
}
