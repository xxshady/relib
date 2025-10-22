use std::alloc::{GlobalAlloc, Layout};
use main_contract::StableLayout;
use crate::gen_imports;

struct Proxy;

unsafe impl GlobalAlloc for Proxy {
  unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
    unsafe {
      gen_imports::proxy_alloc(StableLayout {
        size: layout.size(),
        align: layout.align(),
      })
    }
  }

  unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
    unsafe {
      gen_imports::proxy_dealloc(
        ptr,
        StableLayout {
          size: layout.size(),
          align: layout.align(),
        },
      )
    }
  }
}

#[global_allocator]
static PROXY: Proxy = Proxy;
