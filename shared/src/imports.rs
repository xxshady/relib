use crate::{ModuleId, SliceAllocatorOp, StableLayout, Str};

#[expect(non_camel_case_types)]
pub trait ___Internal___Imports___ {
  fn on_alloc(module: ModuleId, ptr: *mut u8, layout: StableLayout);
  fn on_cached_allocs(module: ModuleId, ops: SliceAllocatorOp);
  fn unrecoverable(module: ModuleId, message: Str) -> !;
  fn is_ptr_allocated(module: ModuleId, ptr: *mut u8) -> bool;
}
