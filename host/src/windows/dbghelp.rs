use std::{
  env::current_exe,
  ffi::c_void,
  path::{Path, PathBuf},
  sync::{Mutex, MutexGuard},
};

use libloading::Library;
use minhook::MinHook;

use crate::{
  helpers::{cstr_bytes, is_library_loaded, path_to_str},
  windows::{
    get_current_dylib,
    imports::{GetCurrentProcess, GetLastError, FALSE, TRUE},
  },
};
use super::{
  imports::{BOOL, HANDLE, PCWSTR, PWSTR},
  str_to_wide_cstring,
};

const STATIC_SEARCH_PATH_ENTRIES: usize = 2;

static INSTANCE: Mutex<Option<Dbghelp>> = Mutex::new(None);

// dbghelp.dll is single-threaded, so for a multi-threaded environment
// we need to lock it manually
fn lock_instance() -> MutexGuard<'static, Option<Dbghelp>> {
  INSTANCE.lock().unwrap_or_else(|e| {
    panic!("Failed to lock dbghelp instance: {e}");
  })
}

#[cfg(feature = "unloading")]
type SymUnloadModule64 = unsafe extern "system" fn(process: HANDLE, base: u64) -> BOOL;
type SymInitializeW =
  unsafe extern "system" fn(hprocess: HANDLE, usersearchpath: PCWSTR, finvadeprocess: BOOL) -> BOOL;
type SymSetSearchPathW = unsafe extern "system" fn(hprocess: HANDLE, searchpatha: PCWSTR) -> BOOL;
type SymGetOptions = unsafe extern "system" fn() -> u32;
type SymSetOptions = unsafe extern "system" fn(symoptions: u32) -> u32;
type SymGetSearchPathW =
  unsafe extern "system" fn(hprocess: HANDLE, searchpatha: PWSTR, searchpathlength: u32) -> BOOL;
type SymRefreshModuleList = unsafe extern "system" fn(process: HANDLE) -> BOOL;

pub struct Dbghelp {
  _lib: Library,
  search_path_entries: Vec<String>,

  refresh_module_list: SymRefreshModuleList,
  set_search_path: SymSetSearchPathW,
  #[cfg(feature = "unloading")]
  unload_module: SymUnloadModule64,
}

pub fn try_init_from_load_module() {
  let mut instance = lock_instance();
  if instance.is_some() {
    return;
  }

  if is_library_loaded("dbghelp.dll") {
    panic!(
      "dbghelp.dll must not be loaded before any module is loaded \
      for backtraces to work correctly on Windows, make sure you don't create backtraces \
      before calling `relib_host::load_module`\n\
      note: if you really need to create backtraces before loading modules consider using `relib_host::init`"
    );
  }

  *instance = Some(unsafe { init() });
}

pub fn try_init() {
  let mut instance = lock_instance();
  if instance.is_some() {
    return;
  }

  if is_library_loaded("dbghelp.dll") {
    panic!(
      "dbghelp.dll must not be loaded before calling `relib_host::init` \
      make sure you don't create backtraces before it"
    );
  }

  *instance = Some(unsafe { init() });
}

unsafe fn init() -> Dbghelp {
  let lib = libloading::Library::new("dbghelp.dll").unwrap_or_else(|e| {
    panic!("Failed to load dbghelp.dll which is needed for backtraces to work correctly: {e}");
  });

  macro_rules! get_lib {
    ($name:ty) => {{
      let symbol: $name = *lib.get(&cstr_bytes(stringify!($name))).unwrap_or_else(|_| {
        panic!(
          "Failed to get {} symbol from dbghelp.dll",
          stringify!($name)
        );
      });
      symbol
    }};
  }

  let initialize: SymInitializeW = {
    let orig = get_lib!(SymInitializeW);

    unsafe extern "system" fn hook(
      _hprocess: HANDLE,
      _usersearchpath: PCWSTR,
      _finvadeprocess: BOOL,
    ) -> BOOL {
      TRUE
    }
    let _type_assert: SymInitializeW = hook;

    let orig = MinHook::create_hook(orig as *mut c_void, hook as *mut c_void).unwrap_or_else(|e| {
      panic!("Failed to hook dbghelp.dll SymInitializeW: {e:?}");
    });
    std::mem::transmute(orig)
  };
  let set_search_path: SymSetSearchPathW = {
    let orig = get_lib!(SymSetSearchPathW);

    unsafe extern "system" fn hook(_hprocess: HANDLE, _searchpatha: PCWSTR) -> BOOL {
      TRUE
    }
    let _type_assert: SymSetSearchPathW = hook;

    let orig = MinHook::create_hook(orig as *mut c_void, hook as *mut c_void).unwrap_or_else(|e| {
      panic!("Failed to hook dbghelp.dll SymSetSearchPathW: {e:?}");
    });
    std::mem::transmute(orig)
  };
  let get_options: SymGetOptions = {
    let orig = get_lib!(SymGetOptions);

    unsafe extern "system" fn hook() -> u32 {
      0
    }
    let _type_assert: SymGetOptions = hook;

    let orig = MinHook::create_hook(orig as *mut c_void, hook as *mut c_void).unwrap_or_else(|e| {
      panic!("Failed to hook dbghelp.dll SymGetOptions: {e:?}");
    });
    std::mem::transmute(orig)
  };
  let set_options: SymSetOptions = {
    let orig = get_lib!(SymSetOptions);

    unsafe extern "system" fn hook(_symoptions: u32) -> u32 {
      0
    }
    let _type_assert: SymSetOptions = hook;

    let orig = MinHook::create_hook(orig as *mut c_void, hook as *mut c_void).unwrap_or_else(|e| {
      panic!("Failed to hook dbghelp.dll SymSetOptions: {e:?}");
    });
    std::mem::transmute(orig)
  };
  let _get_search_path: SymGetSearchPathW = {
    let orig = get_lib!(SymGetSearchPathW);

    unsafe extern "system" fn hook(
      _hprocess: HANDLE,
      _searchpatha: PWSTR,
      _searchpathlength: u32,
    ) -> BOOL {
      FALSE
    }
    let _type_assert: SymGetSearchPathW = hook;

    let orig = MinHook::create_hook(orig as *mut c_void, hook as *mut c_void).unwrap_or_else(|e| {
      panic!("Failed to hook dbghelp.dll SymGetSearchPathW: {e:?}");
    });
    std::mem::transmute(orig)
  };

  let current_options = get_options();
  const SYMOPT_DEFERRED_LOADS: u32 = 0x00000004;
  set_options(current_options | SYMOPT_DEFERRED_LOADS);

  let process = GetCurrentProcess();
  let result = initialize(process, std::ptr::null(), FALSE);
  handle_error(result, "SymInitializeW");

  MinHook::enable_all_hooks().unwrap_or_else(|e| {
    panic!("Failed to enable dbghelp.dll hooks: {e:?}");
  });

  let search_path_entries = {
    let mut entries = vec![".".to_owned()];

    // try blocks when
    (|| {
      let host_path = get_current_dylib().or_else(|| current_exe().ok())?;
      let host_dirname = module_path_to_dirname(&host_path)?;
      let dirname = host_dirname.dirname().to_owned();
      entries.push(dirname);
      Some(())
    })();

    assert_eq!(entries.len(), STATIC_SEARCH_PATH_ENTRIES);

    entries
  };

  let mut instance = Dbghelp {
    refresh_module_list: get_lib!(SymRefreshModuleList),
    set_search_path,
    #[cfg(feature = "unloading")]
    unload_module: get_lib!(SymUnloadModule64),

    _lib: lib,
    search_path_entries,
  };

  refresh_modules_and_search_path(&mut instance);

  instance
}

pub fn add_module(path: &str) {
  let mut instance = lock_instance();
  let instance = instance
    .as_mut()
    .expect("add_module must be called after init");

  if let Some(module_dirname) = module_path_str_to_dirname(path) {
    let dirname = module_dirname.dirname();
    if !instance.search_path_entries.iter().any(|el| el == dirname) {
      instance.search_path_entries.push(dirname.to_owned());
    }
  }

  refresh_modules_and_search_path(instance);
}

#[cfg(feature = "unloading")]
pub fn remove_module(handle: isize, path: &str) -> MutexGuard<'static, Option<Dbghelp>> {
  let mut instance_ = lock_instance();
  let instance = instance_
    .as_mut()
    .expect("remove_module must be called after init");

  if let Some(module_dirname) = module_path_str_to_dirname(path) {
    let dirname = module_dirname.dirname();
    let idx = instance
      .search_path_entries
      .iter()
      .position(|el| el == dirname);
    if let Some(idx) = idx {
      if idx >= STATIC_SEARCH_PATH_ENTRIES {
        instance.search_path_entries.swap_remove(idx);
      }
    } else {
      eprintln!(
        "[relib] warning: couldn't find module to remove it from dbghelp.dll search paths\n\
        module path: {path}"
      );
    }
  }

  let process = unsafe { GetCurrentProcess() };

  let result = unsafe { (instance.unload_module)(process, handle as u64) };
  handle_error(result, "SymUnloadModule64");

  instance_
}

pub fn refresh_modules(mut instance: MutexGuard<'static, Option<Dbghelp>>) {
  // TODO: refactor this
  let instance = instance
    .as_mut()
    .expect("refresh_modules must be called after init");
  refresh_modules_and_search_path(instance);
}

fn handle_error(result: BOOL, fn_name: &str) {
  if result == FALSE {
    let error = unsafe { GetLastError() };
    let error = std::io::Error::from_raw_os_error(error as i32);
    panic!("Something went wrong when calling {fn_name}: {error:?}");
  }
}

fn refresh_modules_and_search_path(instance: &mut Dbghelp) {
  let process = unsafe { GetCurrentProcess() };

  let mut search_path = instance.search_path_entries.join(";");
  search_path += ";";

  // TEST
  println!("[host] search_path: {search_path}");

  let search_path = str_to_wide_cstring(&search_path);

  let result = unsafe { (instance.set_search_path)(process, search_path.as_ptr()) };
  handle_error(result, "SymSetSearchPathW");

  let result = unsafe { (instance.refresh_module_list)(process) };
  handle_error(result, "SymRefreshModuleList");
}

struct ModuleDirname(PathBuf);

impl ModuleDirname {
  fn dirname(&self) -> &str {
    let dirname = self
      .0
      .parent()
      .expect("ModuleDirname guarantees presence of parent");
    path_to_str(dirname)
  }
}

fn module_path_str_to_dirname(path: &str) -> Option<ModuleDirname> {
  let path = Path::new(path);
  module_path_to_dirname(path)
}

fn module_path_to_dirname(path: &Path) -> Option<ModuleDirname> {
  let path = Path::new(path);
  let path = path.canonicalize().unwrap_or_else(|e| {
    panic!("Failed to canonicalize path: {path:?}, reason: {e}");
  });
  path.parent()?;
  Some(ModuleDirname(path))
}
