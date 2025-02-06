use crate::{ModuleId, SliceAllocatorOp, StableLayout, Str};

#[expect(non_camel_case_types)]
pub trait ___Internal___Imports___ {
  fn on_alloc(module: ModuleId, ptr: *mut u8, layout: StableLayout);
  fn on_cached_allocs(module: ModuleId, ops: SliceAllocatorOp);
  fn unrecoverable(message: Str) -> !;

  // for avoiding allocations & thread local creation inside the module
  fn eprintln(message: Str);
}
