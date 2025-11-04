use {
  super::{gen_imports::ModuleImportsImpl, helpers, module_allocs},
  relib_internal_shared::{
    SliceAllocatorOp, StableLayout, Str, imports::___Internal___Imports___ as Imports,
  },
  relib_shared::ModuleId,
};

impl Imports for ModuleImportsImpl {
  fn on_alloc(module: ModuleId, ptr: *mut u8, layout: StableLayout) {
    module_allocs::on_alloc(module, ptr, layout);
  }

  fn on_cached_allocs(module: ModuleId, ops: SliceAllocatorOp) {
    module_allocs::on_cached_allocs(module, ops);
  }

  fn unrecoverable(module: ModuleId, message: Str) -> ! {
    let message = unsafe { message.into_str() };
    helpers::unrecoverable_with_prefix(message, &format!("module id: {module}"));
  }

  fn is_ptr_allocated(module: ModuleId, ptr: *mut u8) -> bool {
    module_allocs::is_ptr_allocated(module, ptr)
  }

  fn transfer_alloc_to_host(module: ModuleId, ptr: *mut u8) -> bool {
    module_allocs::transfer_alloc_to_host(module, ptr)
  }
}
