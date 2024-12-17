use std::{ffi::c_void, mem::transmute};

use crate::helpers::is_it_host_owner_thread;
use super::dtors;

// This function is called when some thread-local registers destructor callback (here it's `dtor`)
#[unsafe(no_mangle)]
pub unsafe extern "C" fn __cxa_thread_atexit_impl(
  dtor: extern "C" fn(*mut c_void),
  obj: *mut c_void,
  dso_symbol: *mut c_void,
) {
  // if we are not in HOST_OWNER_THREAD use original __cxa_thread_atexit_impl
  if !is_it_host_owner_thread() {
    // from fasterthanlime article
    // https://fasterthanli.me/articles/so-you-want-to-live-reload-rust

    type OriginalImpl = extern "C" fn(*mut c_void, *mut c_void, *mut c_void);
    let original_impl: OriginalImpl = transmute(libc::dlsym(
      libc::RTLD_NEXT,
      c"__cxa_thread_atexit_impl".as_ptr(),
    ));

    let dtor = dtor as *mut libc::c_void;
    original_impl(dtor, obj, dso_symbol);
  }
  // otherwise use custom implementation so we can unload them when we
  // no longer need this dynamic library to be loaded
  else {
    // from std (kind of) https://github.com/rust-lang/rust/blob/f6e511eec7342f59a25f7c0534f1dbea00d01b14/library/std/src/sys/thread_local/destructors/linux_like.rs#L53

    // not sure about this transmute (there is transmute in the opposite direction
    // from u8 to c_void in std code so I thought it should also be fine to do it in reverse)
    let dtor = transmute::<extern "C" fn(*mut c_void), extern "C" fn(*mut u8)>(dtor);
    dtors::register(obj.cast(), dtor);
  }
}
