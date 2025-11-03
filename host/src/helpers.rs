use {
  libloading::{Library, Symbol},
  relib_shared::ModuleId,
  std::{
    mem::MaybeUninit,
    path::Path,
    sync::{
      Mutex,
      atomic::{AtomicU64, Ordering},
    },
  },
};

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

pub fn open_library(path: &Path) -> Result<libloading::Library, crate::LoadError> {
  #[cfg(target_os = "linux")]
  let library = {
    use {
      libc::{RTLD_DEEPBIND, RTLD_LAZY, RTLD_LOCAL},
      libloading::os::unix::Library,
    };

    // RTLD_DEEPBIND allows replacing __cxa_thread_atexit_impl (it's needed to call destructors of thread-locals)
    // as well as mmap functions (to unmap leaked mappings) and thread spawn function (to check detached threads)
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
  let fn_ = unsafe { library.get(&cstr_bytes(name)) }?;
  Ok(fn_)
}

// call module export without args with panic handling
// (in case of panic exported function returns false and return value remains uninitialized)
pub unsafe fn call_module_pub_export<R>(
  library: &Library,
  name: &str,
) -> Result<Option<R>, libloading::Error>
// where
//   R: Clone,
{
  // !!! keep in sync with relib_interface crate !!!

  let mangled_name = format!("__relib__{name}");

  let fn_ = unsafe { get_library_export(library, &mangled_name) }?;
  let fn_: Symbol<fn(*mut bool) -> MaybeUninit<R>> = fn_;

  let mut ____success____ = MaybeUninit::<bool>::uninit();

  let return_value = fn_(____success____.as_mut_ptr());

  // SAFETY: this bool is guaranteed to be initialized by the module
  let success = unsafe { ____success____.assume_init() };
  if !success {
    return Ok(None);
  }

  // SAFETY: function returned true so we are allowed to read the pointer
  let return_value = unsafe { return_value.assume_init() };

  Ok(Some(return_value))
}

#[cfg(target_os = "linux")]
mod linux_impl {
  use std::{
    fs::File,
    io::{BufRead, BufReader},
  };

  pub fn is_library_loaded(library_path: &str) -> bool {
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

#[cfg(target_os = "windows")]
mod windows_impl {
  use crate::windows::{
    imports::{GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT, GetModuleHandleExW},
    str_to_wide_cstring,
  };

  pub fn is_library_loaded(library_path: &str) -> bool {
    let library_path = str_to_wide_cstring(library_path);

    let mut module = std::ptr::null_mut();
    let flags = GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT;
    let r = unsafe { GetModuleHandleExW(flags, library_path.as_ptr(), &mut module) };
    r != 0
  }
}

#[cfg(target_os = "linux")]
pub use linux_impl::is_library_loaded;
#[cfg(target_os = "windows")]
pub use windows_impl::is_library_loaded;

pub fn path_to_str(path: &Path) -> &str {
  path.to_str().expect("library path must be UTF-8 string")
}

pub static LIBRARY_LOADING_GUARD: Mutex<LibraryLoadingGuard> = Mutex::new(LibraryLoadingGuard);
pub struct LibraryLoadingGuard;
