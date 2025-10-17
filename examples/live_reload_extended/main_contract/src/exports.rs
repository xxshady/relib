use crate::StableLayout;

pub trait Exports {
  fn main_contract_build_id() -> u128;

  fn call_alloc(layout: StableLayout) -> *mut u8;
  fn call_dealloc(ptr: *mut u8, layout: StableLayout);
}
