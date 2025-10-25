use std::{
  alloc::{GlobalAlloc, Layout},
  sync::OnceLock,
};
use main_contract::{Alloc, Dealloc};

pub struct Proxy {
  pub alloc: OnceLock<Alloc>,
  pub dealloc: OnceLock<Dealloc>,
}

unsafe impl GlobalAlloc for Proxy {
  unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
    let alloc = self.alloc.get().unwrap();
    unsafe { alloc(layout.into()) }
  }

  unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
    let dealloc = self.dealloc.get().unwrap();
    unsafe { dealloc(ptr, layout.into()) }
  }
}

#[global_allocator]
pub static ALLOC_PROXY: Proxy = Proxy {
  alloc: OnceLock::new(),
  dealloc: OnceLock::new(),
};
