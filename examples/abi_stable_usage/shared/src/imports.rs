use std::ffi::CString;

use abi_stable::std_types::RVec;

pub trait Imports {
  fn return_ptr() -> *const CString;
  fn call_drop();

  fn return_ptr2() -> *mut CString;
  fn call_drop2(ptr: *mut CString);

  fn old_foo() -> RVec<u8>;
}
