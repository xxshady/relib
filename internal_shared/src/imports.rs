use {
  crate::{SliceAllocatorOp, StableLayout, Str},
  relib_shared::ModuleId,
};

#[expect(non_camel_case_types)]
pub trait ___Internal___Imports___ {
  fn on_alloc(module: ModuleId, ptr: *mut u8, layout: StableLayout);
  fn on_cached_allocs(module: ModuleId, ops: SliceAllocatorOp);
  fn unrecoverable(module: ModuleId, message: Str) -> !;
  fn is_ptr_allocated(module: ModuleId, ptr: *mut u8) -> bool;
  fn transfer_alloc_to_host(module: ModuleId, ptr: *mut u8) -> bool;
}
