use crate::StableLayout;

pub trait Exports {
  fn call_alloc(layout: StableLayout) -> *mut u8;
  fn call_dealloc(ptr: *mut u8, layout: StableLayout);
}
