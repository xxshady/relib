use {
  super::helpers::unrecoverable,
  minhook::MinHook,
  std::{
    ffi::{c_int, c_void},
    sync::atomic::{AtomicBool, Ordering::Relaxed},
  },
};
use crate::unloading_core::windows_dll_main::DLL_PROCESS_DETACH;

// SAFETY: will be set from main thread and be read too
static mut DEALLOC_CALLBACK: *const c_void = std::ptr::null();

// SAFETY: will be set from main thread and be read too
static mut OBSERVER_DROP_CALLED: bool = false;

static ATEXIT_HOOK_CALLED: AtomicBool = AtomicBool::new(false);
static IS_PROCESS_TERMINATING: AtomicBool = AtomicBool::new(false);

pub fn init() {
  unsafe extern "C" {
    pub fn atexit(cb: unsafe extern "C" fn()) -> c_int;
  }

  // Microsoft-specific DLL behavior of atexit: When a DLL unloads, after DllMain
  // receives DLL_PROCESS_DETACH, the DLL's atexit callbacks run in reverse
  // registration order, with the last callback registered running first.

  // TODO: SAFETY
  unsafe {
    atexit(call_dealloc_callback);
  }

  type AtExitFn = unsafe extern "C" fn(cb: unsafe extern "C" fn()) -> c_int;

  static mut ORIG_ATEXIT: AtExitFn = atexit;

  unsafe extern "C" fn atexit_hook(cb: unsafe extern "C" fn()) -> c_int {
    ATEXIT_HOOK_CALLED.store(true, Relaxed);

    // TODO: SAFETY
    unsafe { ORIG_ATEXIT(cb) }
  }

  let orig_atexit =
    unsafe { MinHook::create_hook(atexit as *mut c_void, atexit_hook as *mut c_void) };
  let orig_atexit = orig_atexit.unwrap_or_else(|_| {
    unrecoverable("failed to hook atexit");
  });

  // TODO: SAFETY
  unsafe {
    ORIG_ATEXIT = std::mem::transmute::<*mut c_void, AtExitFn>(orig_atexit);
  }

  // TODO: SAFETY
  unsafe {
    MinHook::enable_all_hooks().unwrap_or_else(|_| {
      unrecoverable("MinHook::enable_all_hooks failed");
    });
  }

  init_tls_destructors_in_std();
}

extern "C" fn call_dealloc_callback() {
  if IS_PROCESS_TERMINATING.load(Relaxed) {
    return;
  }

  unsafe {
    if !OBSERVER_DROP_CALLED {
      unrecoverable(
        "std must call thread local destructors using atexit before this atexit callback when this DLL unloads",
      );
    }

    if DEALLOC_CALLBACK.is_null() {
      unrecoverable(
        "dealloc callback must be set before library close call (ataxit callback of module)",
      );
    }

    // SAFETY: see dealloc_callback definition in host windows_dealloc module
    let callback: extern "C" fn() = std::mem::transmute(DEALLOC_CALLBACK);
    callback();
  }
}

fn init_tls_destructors_in_std() {
  #[expect(dead_code)]
  struct Observer(Vec<u8>);

  impl Drop for Observer {
    fn drop(&mut self) {
      unsafe {
        OBSERVER_DROP_CALLED = true;
      }
    }
  }

  thread_local! {
    static OBSERVER: Observer = Observer(vec![1]);
  }

  // we can't be completely sure that atexit will be called here exactly by
  // thread local implementation of std but it's still better than nothing
  // https://github.com/rust-lang/rust/blob/51c768aa5cb99ae7670e97b0e8392f926e79855e/library/std/src/sys/thread_local/guard/windows.rs#L163

  if ATEXIT_HOOK_CALLED.load(Relaxed) {
    unrecoverable("atexit hook must not be called before std tls destructor initialization");
  }

  // initialize it:
  OBSERVER.with(|_| {});

  if !ATEXIT_HOOK_CALLED.load(Relaxed) {
    unrecoverable("atexit hook must be called during std tls destructor initialization");
  }
}

pub unsafe fn set_dealloc_callback(callback: *const c_void) {
  unsafe {
    DEALLOC_CALLBACK = callback;
  }
}

pub fn dll_main(reason: u32, lpv_reserved: *mut c_void) {
  if reason == DLL_PROCESS_DETACH && !lpv_reserved.is_null() {
    IS_PROCESS_TERMINATING.store(true, Relaxed);
  }
}
