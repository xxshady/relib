use crate::{allocator_lock, gen_imports, HOST_OWNER_THREAD};

pub fn unrecoverable(message: &str) -> ! {
  unsafe { gen_imports::unrecoverable(message.into()) }
}

pub fn assert_allocator_is_still_accessible() {
  if allocator_lock() &&
  // allow access to allocator if it's accessed from HOST_OWNER_THREAD, 
  // here we assume that it can be called from destructors of thread-locals on linux, so we allow it
  !is_it_host_owner_thread()
  {
    unrecoverable(
      "module allocator was invoked while module was in the process of unloading\n\
      note: before unloading the module, make sure that all threads are joined if any were spawned by it\n\
      note: you can register \"before_unload\" callback for it",
    );
  }
}

pub fn is_it_host_owner_thread() -> bool {
  thread_id::get() == unsafe { HOST_OWNER_THREAD }
}

/// on windows thread-local destructors are called automatically in `library.close()`,
/// but at that time it's already too late, because we deallocate everything before
/// `library.close()` (in "exit" module export) so we need to turn allocator methods into no-ops
/// (unlike linux where we call thread-local destructors ourselves, see `thread_locals` module)
#[cfg(target_os = "windows")]
pub fn disable_allocator_for_thread_local_destructors() -> bool {
  allocator_lock()
}
