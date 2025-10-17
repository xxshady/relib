use std::{
  collections::HashMap,
  ffi::c_void,
  sync::{LazyLock, Mutex, MutexGuard, Once},
};

use minhook::MinHook;

use crate::{
  module::WindowsLibraryHandle,
  windows::{
    get_dylib_handle_from_addr,
    imports::{CreateThread, HANDLE},
  },
};
use super::{helpers::unrecoverable};

type Payload = (
  unsafe extern "system" fn(main: *mut c_void) -> u32,
  *mut c_void,
  WindowsLibraryHandle,
);

static mut ORIG: CreateThread = CreateThread;

static INIT: Once = Once::new();

type ModuleThreadsCount = HashMap<WindowsLibraryHandle, u64>;

static MODULE_THREADS: LazyLock<Mutex<ModuleThreadsCount>> = LazyLock::new(Default::default);

fn lock_module_threads() -> MutexGuard<'static, ModuleThreadsCount> {
  let Ok(module_threads) = MODULE_THREADS.lock() else {
    unrecoverable("failed to lock MODULE_THREADS");
  };

  module_threads
}

pub fn add_module(module_handle: WindowsLibraryHandle) {
  let mut module_threads = lock_module_threads();
  module_threads.insert(module_handle, 0);
}

pub fn remove_module(module_handle: WindowsLibraryHandle) -> Result<(), ()> {
  let mut module_threads = lock_module_threads();
  let Some(threads) = module_threads.remove(&module_handle) else {
    // TEST
    // TODO: use unrecoverable instead?

    let backtrace = std::backtrace::Backtrace::force_capture();
    eprintln!("backtrace: {backtrace:#}");
    panic!("Failed to remove module_threads of module with handle: {module_handle}");
  };

  if threads == 0 { Ok(()) } else { Err(()) }
}

pub unsafe fn init() {
  INIT.call_once(|| {
    let orig = unsafe { MinHook::create_hook(CreateThread as *mut c_void, hook as *mut c_void) };
    let orig = orig.unwrap_or_else(|e| {
      panic!("Failed to hook CreateThread: {e:?}");
    });

    // SAFETY: we are only writing to it once in this synchronized closure by std::sync::Once
    unsafe {
      ORIG = std::mem::transmute::<*mut c_void, CreateThread>(orig);
    }
  });
}

unsafe extern "system" fn hook(
  lpthreadattributes: *const c_void,
  dwstacksize: usize,
  lpstartaddress: unsafe extern "system" fn(main: *mut c_void) -> u32,
  lpparameter: *mut c_void,
  dwcreationflags: u32,
  lpthreadid: *mut u32,
) -> HANDLE {
  let module_handle = unsafe { get_dylib_handle_from_addr(lpstartaddress as *const _) };
  if let Some(module_handle) = module_handle {
    let module_handle = module_handle as WindowsLibraryHandle;

    lock_module_threads()
      .entry(module_handle)
      .and_modify(|count| {
        *count += 1;
      });

    let payload: Payload = (lpstartaddress, lpparameter, module_handle);
    let payload = Box::new(payload);
    let payload = Box::into_raw(payload);

    unsafe {
      ORIG(
        lpthreadattributes,
        dwstacksize,
        thread_start,
        payload as *mut c_void,
        dwcreationflags,
        lpthreadid,
      )
    }

  // TODO: check if this branch is ever executed
  } else {
    unsafe {
      ORIG(
        lpthreadattributes,
        dwstacksize,
        lpstartaddress,
        lpparameter,
        dwcreationflags,
        lpthreadid,
      )
    }
  }
}

const _TYPE_ASSERT: CreateThread = hook;

unsafe extern "system" fn thread_start(payload: *mut c_void) -> u32 {
  // TODO: SAFETY
  let (ret, module_handle) = unsafe {
    let (f, value, module_handle) = *Box::from_raw(payload as *mut Payload);

    (f(value), module_handle)
  };

  lock_module_threads()
    .entry(module_handle)
    .and_modify(|count| {
      *count -= 1;
    });

  ret
}
