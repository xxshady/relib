use {
  crate::{Module, ModuleExportsForHost, module_allocs, unloading::helpers::unrecoverable},
  std::{cell::RefCell, ffi::c_void},
};

thread_local! {
  static DEALLOC_CLOSURE: RefCell<Option<Box<dyn FnOnce()>>> = Default::default();
}

pub extern "C" fn dealloc_callback() {
  DEALLOC_CLOSURE.with_borrow_mut(|v| {
    let callback = v.take().unwrap_or_else(|| {
      unrecoverable("DEALLOC_CLOSURE must be set when dealloc_callback is called");
    });
    callback();
  });
}

pub fn set<E: ModuleExportsForHost>(module: Module<E>, library_path: String) {
  unsafe {
    module
      .internal_exports
      .set_dealloc_callback(dealloc_callback as *const c_void);
  }

  // !!! don't try to synchronize with other threads in this callback !!!
  // https://learn.microsoft.com/en-us/windows/win32/dlls/dynamic-link-library-best-practices#general-best-practices
  DEALLOC_CLOSURE.set(Some(Box::new(move || {
    unsafe {
      module.internal_exports.lock_module_allocator();
    }
    module_allocs::remove_module(
      module.id,
      &module.internal_exports,
      &library_path,
      module.alloc_tracker_enabled,
    );
  })));
}

pub fn successfully_called() -> bool {
  DEALLOC_CLOSURE.with(|c| c.borrow().is_none())
}
