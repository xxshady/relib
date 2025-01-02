use abi_stable::std_types::RString;

pub trait Exports {
  fn a() -> i32;
  fn b(r: &RString) -> RString;
}
