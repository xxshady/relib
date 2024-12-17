use std::sync::atomic::{AtomicU64, Ordering};

static SPAWNED_THREADS_COUNT: AtomicU64 = AtomicU64::new(0);

pub fn spawned_threads_count() -> u64 {
  SPAWNED_THREADS_COUNT.load(Ordering::Relaxed)
}

type Payload = (
  extern "C" fn(*mut libc::c_void) -> *mut libc::c_void,
  *mut libc::c_void,
);

#[unsafe(no_mangle)]
unsafe extern "C" fn pthread_create(
  native: *mut libc::pthread_t,
  attr: *const libc::pthread_attr_t,
  f: extern "C" fn(*mut libc::c_void) -> *mut libc::c_void,
  value: *mut libc::c_void,
) -> libc::c_int {
  let payload: Payload = (f, value);
  let payload = Box::new(payload);
  let payload = Box::into_raw(payload);

  type OriginalImpl = extern "C" fn(
    native: *mut libc::pthread_t,
    attr: *const libc::pthread_attr_t,
    f: extern "C" fn(*mut libc::c_void) -> *mut libc::c_void,
    value: *mut libc::c_void,
  ) -> libc::c_int;

  let original_impl: OriginalImpl =
    std::mem::transmute(libc::dlsym(libc::RTLD_NEXT, c"pthread_create".as_ptr()));

  let result = original_impl(native, attr, thread_start, payload as *mut libc::c_void);

  if result == 0 {
    SPAWNED_THREADS_COUNT.fetch_add(1, Ordering::Relaxed);
  }

  result
}

extern "C" fn thread_start(payload: *mut libc::c_void) -> *mut libc::c_void {
  let ret = unsafe {
    let (f, value) = *Box::from_raw(payload as *mut Payload);
    f(value)
  };

  SPAWNED_THREADS_COUNT.fetch_sub(1, Ordering::Relaxed);

  ret
}
