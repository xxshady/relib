use crate::StableLayout;

pub trait Imports {
  fn foo() -> i32;

  fn alloc(layout: StableLayout) -> *mut u8;
  fn dealloc(ptr: *mut u8, layout: StableLayout);
}
