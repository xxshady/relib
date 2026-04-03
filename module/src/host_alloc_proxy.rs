use {
  relib_internal_shared::{Alloc, Dealloc},
  std::{
    alloc::{GlobalAlloc, Layout},
    sync::OnceLock,
  },
};

/// [`GlobalAlloc`] that uses global allocator of the host.
pub struct HostAllocProxy;

unsafe impl GlobalAlloc for HostAllocProxy {
  unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
    let alloc = HOST_ALLOC_PROXY.alloc.get().unwrap();
    unsafe { alloc(layout.into()) }
  }

  unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
    let dealloc = HOST_ALLOC_PROXY.dealloc.get().unwrap();
    unsafe { dealloc(ptr, layout.into()) }
  }
}

pub(crate) struct AllocProxy {
  pub alloc: OnceLock<Alloc>,
  pub dealloc: OnceLock<Dealloc>,
}

pub(crate) static HOST_ALLOC_PROXY: AllocProxy = AllocProxy {
  alloc: OnceLock::new(),
  dealloc: OnceLock::new(),
};
