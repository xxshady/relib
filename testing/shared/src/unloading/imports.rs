use abi_stable::std_types::{RStr, RString};

pub trait Imports {
  fn a() -> i32;
  fn b(r: RStr) -> RString;
  fn b2(r: RStr, r2: RStr) -> RString;
  fn d();
  fn ptr() -> *const i32;
}
