use {
  crate::{SliceAllocatorOp, StableLayout, Str},
};

#[expect(non_camel_case_types)]
pub trait ___Internal___Imports___ {
  fn on_alloc(ptr: *mut u8, layout: StableLayout);
  fn on_cached_allocs(ops: SliceAllocatorOp);
  fn unrecoverable(message: Str) -> !;
  fn is_ptr_allocated(ptr: *mut u8) -> bool;
  fn transfer_alloc_to_host(ptr: *mut u8) -> bool;
}
