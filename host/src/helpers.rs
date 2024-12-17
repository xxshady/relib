use std::{
  mem::MaybeUninit,
  path::Path,
  sync::atomic::{AtomicU64, Ordering},
};

use relib_internal_shared::ModuleId;
use libloading::{Library, Symbol};

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
  eprintln!("aborting");
  std::process::abort();
}

pub fn cstr_bytes(str: &str) -> Vec<u8> {
  [str.as_bytes(), &[0]].concat()
}

pub fn next_module_id() -> ModuleId {
  // module ids start from 1
  static ID_COUNTER: AtomicU64 = AtomicU64::new(1);

  let id = ID_COUNTER.fetch_add(1, Ordering::SeqCst);
  assert_ne!(id, 0, "this must never happen (integer overflow)");
  id
}

pub fn open_library(path: &Path) -> Result<libloading::Library, crate::Error> {
  #[cfg(target_os = "linux")]
  let library = {
    use libloading::os::unix::Library;
    use libc::{RTLD_DEEPBIND, RTLD_LAZY, RTLD_LOCAL};

    // RTLD_DEEPBIND allows replacing __cxa_thread_atexit_impl (it's needed to call destructors of thread-locals)
    // only for dynamic library without replacing it for the whole executable
    const FLAGS: i32 = RTLD_LAZY | RTLD_LOCAL | RTLD_DEEPBIND;

    unsafe { Library::open(Some(path), FLAGS) }?.into()
  };

  #[cfg(target_os = "windows")]
  let library = {
    use libloading::os::windows::Library;
    unsafe { Library::new(path) }?.into()
  };

  Ok(library)
}

pub unsafe fn get_library_export<'lib, F>(
  library: &'lib Library,
  name: &str,
) -> Result<Symbol<'lib, F>, libloading::Error> {
  let fn_ = library.get(&cstr_bytes(name))?;
  Ok(fn_)
}

// call module export with panic handling
// (in case of panic exported function returns false and return value remains uninitialized)
pub unsafe fn call_module_pub_export<R>(
  library: &Library,
  name: &str,
) -> Result<Option<R>, libloading::Error> {
  let fn_ = get_library_export(library, name)?;
  let fn_: Symbol<extern "C" fn(*mut MaybeUninit<R>) -> bool> = fn_;

  let mut return_value = MaybeUninit::uninit();

  let success = fn_(&mut return_value);
  if !success {
    return Ok(None);
  }

  // SAFETY: function returned true so we are allowed to read the pointer
  let return_value = return_value.assume_init_read();
  Ok(Some(return_value))
}

#[cfg(target_os = "linux")]
pub mod linux {
  use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
  };

  pub fn is_library_loaded(library_path: &Path) -> bool {
    let library_path = library_path
      .to_str()
      .expect("library path must be UTF-8 string");

    let file = File::open("/proc/self/maps").expect("Failed to open /proc/self/maps");
    let reader = BufReader::new(file);

    reader.lines().any(|line_result| {
      if let Ok(line) = line_result {
        line.contains(library_path)
      } else {
        false
      }
    })
  }
}
