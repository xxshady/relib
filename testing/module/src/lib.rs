mod shared;

#[cfg(feature = "unloading")]
mod unloading;
#[cfg(feature = "no_unloading")]
mod no_unloading;

#[cfg(feature = "exportify")]
mod exportify;

#[cfg(feature = "threads_check")]
mod threads_check;

#[cfg(feature = "before_unload_panic")]
mod before_unload_panic;

#[cfg(feature = "code_change")]
mod code_change;

#[cfg(feature = "multiple_modules")]
mod multiple_modules;

#[cfg(any(
  feature = "panic_in_interface_module",
  feature = "panic_in_interface_host"
))]
mod panic_in_interface;

#[cfg(feature = "backtrace_unloading")]
mod backtrace_unloading;

#[cfg(feature = "is_already_loaded_error")]
mod is_already_loaded_error;

#[cfg(feature = "dbghelp_is_already_loaded_init")]
mod dbghelp_is_already_loaded_init;

#[cfg(feature = "backtrace_unloading_host_as_dylib")]
mod backtrace_unloading_host_as_dylib;
