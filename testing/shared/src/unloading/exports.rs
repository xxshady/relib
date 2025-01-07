use abi_stable::std_types::{RStr, RString};

pub trait Exports {
  fn a() -> i32;
  fn b(r: RStr) -> RString;
  fn d();
}
