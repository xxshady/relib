use std::{
  ffi::{c_int, c_uint, c_void},
  sync::{
    atomic::{AtomicBool, Ordering::Relaxed},
    Mutex, MutexGuard,
  },
};

use crate::unloading_core::helpers::unrecoverable;

#[expect(non_camel_case_types)]
type pthread_key_t = c_uint;
type Dtor = Option<unsafe extern "C" fn(*mut c_void)>;
type Store = Vec<(pthread_key_t, Dtor)>;

static STORE: Mutex<Store> = Mutex::new(Vec::new());

pub static EXIT_DELETE: AtomicBool = AtomicBool::new(false);

pub fn lock_store() -> MutexGuard<'static, Store> {
  STORE.lock().unwrap_or_else(|_| {
    unrecoverable("failed to lock pthread keys store");
  })
}

#[unsafe(no_mangle)]
unsafe extern "C" fn pthread_key_create(key: *mut pthread_key_t, dtor: Dtor) -> c_int {
  type OriginalImpl = unsafe extern "C" fn(key: *mut pthread_key_t, dtor: Dtor) -> c_int;

  unsafe {
    let original_impl: OriginalImpl =
      std::mem::transmute(libc::dlsym(libc::RTLD_NEXT, c"pthread_key_create".as_ptr()));

    let result = original_impl(key, dtor);

    if result == 0 {
      lock_store().push((*key, dtor));
    }

    result
  }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_key_delete(key: pthread_key_t) -> c_int {
  {
    if !EXIT_DELETE.load(Relaxed) {
      let mut store = lock_store();
      let idx = store.iter().position(|&(key_, _)| key_ == key);
      if let Some(idx) = idx {
        store.swap_remove(idx);
      }
    }
  }

  type OriginalImpl = unsafe extern "C" fn(key: pthread_key_t) -> c_int;

  // TODO: SAFETY
  unsafe {
    let original_impl: OriginalImpl =
      std::mem::transmute(libc::dlsym(libc::RTLD_NEXT, c"pthread_key_delete".as_ptr()));

    original_impl(key)
  }
}

pub fn cleanup() {
  // TODO: use mutex instead?
  EXIT_DELETE.store(true, Relaxed);

  let pthread_keys = std::mem::take(&mut *lock_store());
  // TODO: call dtor instead?
  for (key, _dtor) in pthread_keys {
    let r = unsafe { pthread_key_delete(key) };
    if r != 0 {
      unrecoverable("libc::pthread_key_delete failed");
    }
  }
}
