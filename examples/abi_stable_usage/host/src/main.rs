mod alloc_counter;

use std::{cell::Cell, ffi::CString, sync::atomic::Ordering::Relaxed};

use abi_stable::{std_types::RVec, traits::IntoReprC};
use alloc_counter::{ALLOCS, DEALLOC_TRACES, IGNORE, STORE_TRACES};
use shared::imports::Imports;

relib_interface::include_exports!();
use gen_exports::ModuleExports;

relib_interface::include_imports!();
use gen_imports::{init_imports, ModuleImportsImpl};

thread_local! {
  static RETURN_DESTRUCTOR: Cell<Option<Box<dyn FnOnce()>>> = Cell::new(None);
}

impl Imports for ModuleImportsImpl {
  fn return_ptr() -> *const CString {
    let value = CString::new("some memory").unwrap();
    let ptr = Box::into_raw(Box::new(value));

    assert!(RETURN_DESTRUCTOR.take().is_none());
    RETURN_DESTRUCTOR.set(Some(Box::new(move || unsafe {
      drop(Box::from_raw(ptr));
    })));
    ptr
  }

  fn call_drop() {
    RETURN_DESTRUCTOR.take().unwrap()();
  }

  fn return_ptr2() -> *mut CString {
    let value = CString::new("some memory").unwrap();
    Box::into_raw(Box::new(value))
  }

  fn call_drop2(ptr: *mut CString) {
    unsafe {
      drop(Box::from_raw(ptr));
    }
  }

  fn old_foo() -> RVec<u8> {
    vec![1_u8; 1024 * 1024 * 100].into()
  }
}

fn main() {
  let path_to_dylib = if cfg!(target_os = "linux") {
    "target/debug/libmodule.so"
  } else {
    "target/debug/module.dll"
  };

  let module = relib_host::load_module::<ModuleExports>(path_to_dylib, init_imports)
    .unwrap_or_else(|e| {
      panic!("module loading failed: {e:#}");
    });

  unsafe { module.call_main::<()>() }.unwrap();

  let bar_value = unsafe { module.exports().bar() }.unwrap();
  let string = bar_value.clone();
  dbg!(string);

  // module.unload() is provided when unloading feature of relib_host crate is enabled
  #[cfg(feature = "unloading")]
  {
    println!("unloading feature is enabled, calling module unload");

    module.unload().unwrap_or_else(|e| {
      panic!("module unloading failed: {e:#}");
    });
  }
}
