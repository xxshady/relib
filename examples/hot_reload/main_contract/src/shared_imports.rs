use crate::StableLayout;

// these imports are shared between main module and update module
pub trait SharedImports {
  fn foo() -> i32;

  // only called by update module from proxy allocator
  fn proxy_alloc(layout: StableLayout) -> *mut u8;
  fn proxy_dealloc(ptr: *mut u8, layout: StableLayout);
}
