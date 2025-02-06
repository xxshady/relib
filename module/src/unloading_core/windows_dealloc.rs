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

use super::helpers::unrecoverable;

#[expect(clippy::upper_case_acronyms)]
type BOOL = i32;
const DLL_PROCESS_DETACH: u32 = 0;
const TRUE: i32 = 1;

static SUPER_SPECIAL_CALLBACK_CALLED: AtomicBool = AtomicBool::new(false);

// SAFETY: will be set from main thread and be read too
static mut DEALLOC_CALLBACK: *const c_void = std::ptr::null();

// TODO: check if its called for other dlls as well
#[unsafe(no_mangle)]
extern "system" fn DllMain(_: *mut c_void, reason: u32, _: *mut c_void) -> BOOL {
  if reason != DLL_PROCESS_DETACH {
    return TRUE;
  }

  // use std::io::Write;
  // let mut buffer = [0u8; 32];
  // // let msg = {
  // let mut writer = &mut buffer[..];
  // writeln!(writer, "DllMain reason: {}", reason).unwrap();
  // let mut stdout = std::io::stdout().lock();
  // stdout.write_all(&buffer).unwrap();
  // stdout.flush().unwrap();

  if !SUPER_SPECIAL_CALLBACK_CALLED.load(Relaxed) {
    unrecoverable("super special callback was not called before DLL_PROCESS_DETACH :c");
  }

  unsafe {
    if DEALLOC_CALLBACK.is_null() {
      unrecoverable("dealloc callback was not set before DLL_PROCESS_DETACH :c");
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
#[link_section = ".CRT$XLB"]
#[used]
static SUPER_SPECIAL_CALLBACK: extern "system" fn(*mut c_void, u32, *mut c_void) =
  callback_to_ensure_tls_destructors_are_called_before_dllmain;

// TODO: check if its called for other dlls as well
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
