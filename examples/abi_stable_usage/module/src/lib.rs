use abi_stable::std_types::{RString, RVec};
use shared::exports::Exports;

relib_interface::include_exports!();
use gen_exports::ModuleExportsImpl;

relib_interface::include_imports!();

impl Exports for ModuleExportsImpl {
  fn bar() -> RString {
    "FFI-safe string!".into()
  }
}

#[relib_module::export]
fn main() {
  println!("hello world");

  // use gen_imports::{return_ptr, call_drop};
  // unsafe {
  //   let ptr = return_ptr();
  //   let cloned = (*ptr).clone();
  //   call_drop();

  //   dbg!(cloned);
  // }

  use gen_imports::{return_ptr2, call_drop2};
  unsafe {
    let ptr = return_ptr2();
    let cloned = (*ptr).clone();
    call_drop2(ptr);

    dbg!(cloned);
  }
}
