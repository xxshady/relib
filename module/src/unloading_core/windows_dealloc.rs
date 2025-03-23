// if in the future DllMain with DLL_PROCESS_DETACH will no longer be called
// after thread-local destructors (see `#[link_section = ".CRT$XLB"]` in the standard library implementation)
// then if standard library still uses pop() when calling destructors (see https://github.com/rust-lang/rust/blob/2f92f050e83bf3312ce4ba73c31fe843ad3cbc60/library/std/src/sys/thread_local/destructors/list.rs#L30)
// and then we can try to define a thread-local with a Drop in which
// we will deallocate cached allocs and that thread-local should be
// initialized before all others (!!!)
//
// I know that this is horrible but I didn't find any better solution,
// let's hope that the current behavior won't change :sadge_pray:

use std::{
  ffi::c_void,
  sync::atomic::{AtomicBool, Ordering::Relaxed},
};

use super::{helpers::unrecoverable, MODULE_ID};

#[expect(clippy::upper_case_acronyms)]
type BOOL = i32;
const DLL_PROCESS_DETACH: u32 = 0;
const TRUE: i32 = 1;

static SUPER_SPECIAL_CALLBACK_CALLED: AtomicBool = AtomicBool::new(false);

// SAFETY: will be set from main thread and be read too
static mut DEALLOC_CALLBACK: *const c_void = std::ptr::null();

#[unsafe(no_mangle)]
unsafe extern "system" fn DllMain(_: *mut c_void, reason: u32, lpv_reserved: *mut c_void) -> BOOL {
  // are we actually initialized?
  // maybe current module was unloaded before initialization
  // (for example, due to compilation info check fail)
  if MODULE_ID == 0 {
    return TRUE;  
  }

  if !(
    reason == DLL_PROCESS_DETACH && 
    // lpv_reserved is null if FreeLibrary has been called or the DLL load failed and non-null
    // if the process is terminating
    lpv_reserved.is_null()
  ) {
    return TRUE;
  }

  if !SUPER_SPECIAL_CALLBACK_CALLED.load(Relaxed) {
    unrecoverable("super special callback was not called before DLL_PROCESS_DETACH");
  }

  unsafe {
    if DEALLOC_CALLBACK.is_null() {
      unrecoverable("dealloc callback was not set before DLL_PROCESS_DETACH");
    }

    // SAFETY: see dealloc_callback in host Module unload() impl
    let callback: extern "system" fn() = std::mem::transmute(DEALLOC_CALLBACK);
    callback();
  }

  TRUE
}

// wtf is this? let me explain it with a wonderful quote from standard library:
// //! # What's up with this callback?
// //!
// //! The callback specified receives a number of parameters from... someone!
// //! (the kernel? the runtime? I'm not quite sure!) There are a few events that
// //! this gets invoked for, but we're currently only interested on when a
// //! thread or a process "detaches" (exits). The process part happens for the
// //! last thread and the thread part happens for any normal thread.
// //!
//
// we are interested in this callback because it's called when module is being unloaded
// and standard library uses it to call thread-local destructors
#[unsafe(link_section = ".CRT$XLB")]
#[used]
static SUPER_SPECIAL_CALLBACK: extern "system" fn(*mut c_void, u32, *mut c_void) =
  callback_to_ensure_tls_destructors_are_called_before_dllmain;

extern "system" fn callback_to_ensure_tls_destructors_are_called_before_dllmain(
  _: *mut c_void,
  reason: u32,
  _: *mut c_void,
) {
  if reason != DLL_PROCESS_DETACH {
    return;
  }

  SUPER_SPECIAL_CALLBACK_CALLED.store(true, Relaxed);
}

pub unsafe fn set_dealloc_callback(callback: *const c_void) {
  DEALLOC_CALLBACK = callback;
}
