use {
  super::{gen_imports::ModuleImportsImpl, helpers, module_allocs},
  relib_internal_shared::{
    ModuleId, SliceAllocatorOp, StableLayout, Str, imports::___Internal___Imports___ as Imports,
  },
};

impl Imports for ModuleImportsImpl {
  fn on_alloc(ptr: *mut u8, layout: StableLayout) {
    module_allocs::on_alloc(unsafe { crate::unloading_core::MODULE_ID }, ptr, layout);
  }

  fn on_cached_allocs(ops: SliceAllocatorOp) {
    module_allocs::on_cached_allocs(unsafe { crate::unloading_core::MODULE_ID }, ops);
  }

  fn unrecoverable(message: Str) -> ! {
    let message = unsafe { message.into_str() };
    helpers::unrecoverable_with_prefix(message, &format!("module id: {}", unsafe { crate::unloading_core::MODULE_ID }));
  }

  fn is_ptr_allocated(ptr: *mut u8) -> bool {
    module_allocs::is_ptr_allocated(unsafe { crate::unloading_core::MODULE_ID }, ptr)
  }

  fn transfer_alloc_to_host(ptr: *mut u8) -> bool {
    module_allocs::transfer_alloc_to_host(unsafe { crate::unloading_core::MODULE_ID }, ptr)
  }
}
