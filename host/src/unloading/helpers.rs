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
  eprintln!("backtrace:\n{backtrace}");

  eprintln!("aborting");
  std::process::abort();
}

#[cfg(target_os = "windows")]
pub mod windows {
  use libloading::{os::windows::Library as WindowsLibrary, Library};
  use crate::module::WindowsLibraryHandle;

  pub fn library_handle(library: Library) -> (Library, WindowsLibraryHandle) {
    let handle = WindowsLibrary::from(library).into_raw();
    let library = unsafe { WindowsLibrary::from_raw(handle) };
    let library = Library::from(library);
    (library, handle)
  }
}
