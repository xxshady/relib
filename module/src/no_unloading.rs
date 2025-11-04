use {
  crate::{HostAllocProxy, host_alloc_proxy::HOST_ALLOC_PROXY},
  relib_internal_shared::{
    Alloc, Dealloc, exports_no_unloading::___Exports___NoUnloading___ as Exports,
  },
};

// TODO: add test for it
/// Modules must use global allocator of the host to safely move allocations between them.
///
/// **Safety requirement** if you want to set your own `#[global_allocator]`:
/// it must use global allocator of the host (you can use [`HostAllocProxy`] for it).
#[cfg(feature = "host_global_alloc")]
#[global_allocator]
static ALLOC: HostAllocProxy = HostAllocProxy;

relib_interface::include_exports!(gen_exports, "internal_generated_module_no_unloading");
use gen_exports::ModuleExportsImpl;

impl Exports for ModuleExportsImpl {
  fn init(alloc: Alloc, dealloc: Dealloc) {
    HOST_ALLOC_PROXY.alloc.set(alloc).unwrap();
    HOST_ALLOC_PROXY.dealloc.set(dealloc).unwrap();
  }
}
