use {
  super::helpers::unrecoverable,
  std::{
    ffi::c_void,
    mem::transmute,
    sync::{Mutex, MutexGuard},
  },
};

struct Ptr(*mut c_void);

// SAFETY: is stored in Mutex and will be passed to libc::munmap on module unloading
unsafe impl Send for Ptr {}
unsafe impl Sync for Ptr {}

static MMAPS: Mutex<Vec<(Ptr, libc::size_t)>> = Mutex::new(Vec::new());

fn lock_mmaps() -> MutexGuard<'static, Vec<(Ptr, libc::size_t)>> {
  MMAPS.lock().unwrap_or_else(|_| {
    unrecoverable("failed to lock MMAPS");
  })
}

#[unsafe(no_mangle)]
unsafe extern "C" fn mmap64(
  addr: *mut c_void,
  len: libc::size_t,
  prot: libc::c_int,
  flags: libc::c_int,
  fd: libc::c_int,
  offset: libc::off64_t,
) -> *mut c_void {
  type OriginalImpl = unsafe extern "C" fn(
    addr: *mut c_void,
    len: libc::size_t,
    prot: libc::c_int,
    flags: libc::c_int,
    fd: libc::c_int,
    offset: libc::off64_t,
  ) -> *mut c_void;

  // TODO: SAFETY
  unsafe {
    let original_impl: OriginalImpl = transmute(libc::dlsym(libc::RTLD_NEXT, c"mmap64".as_ptr()));
    let ptr = original_impl(addr, len, prot, flags, fd, offset);

    if ptr != libc::MAP_FAILED {
      lock_mmaps().push((Ptr(ptr), len));
    }

    ptr
  }
}

#[unsafe(no_mangle)]
unsafe extern "C" fn munmap(addr: *mut c_void, len: libc::size_t) -> libc::c_int {
  {
    let mut mmaps = lock_mmaps();
    let idx = mmaps.iter().position(|(ptr, _)| ptr.0 == addr);
    if let Some(idx) = idx {
      mmaps.swap_remove(idx);
    }
  }

  type OriginalImpl = unsafe extern "C" fn(addr: *mut c_void, len: libc::size_t) -> libc::c_int;

  // TODO: SAFETY
  unsafe {
    let original_impl: OriginalImpl = transmute(libc::dlsym(libc::RTLD_NEXT, c"munmap".as_ptr()));
    original_impl(addr, len)
  }
}

pub fn cleanup() {
  unsafe {
    let mmaps = std::mem::take(&mut *lock_mmaps());
    for (ptr, len) in mmaps {
      let r = libc::munmap(ptr.0, len);

      if r != 0 {
        unrecoverable("libc::munmap failed");
      }
    }
  }
}
