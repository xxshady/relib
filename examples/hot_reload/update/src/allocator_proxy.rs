use std::alloc::{GlobalAlloc, Layout};
use main_contract::StableLayout;
use crate::gen_imports;

struct Proxy;

unsafe impl GlobalAlloc for Proxy {
  unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
    unsafe {
      // if USE_SYSTEM_ALLOC {
      //   return System.alloc(layout);
      // }

      gen_imports::alloc(StableLayout {
        size: layout.size(),
        align: layout.align(),
      })
    }
  }

  unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
    unsafe {
      // if USE_SYSTEM_ALLOC {
      //   return System.dealloc(ptr, layout);
      // }

      gen_imports::dealloc(
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

// TODO: fix this properly
// relib_module
// static mut USE_SYSTEM_ALLOC: bool = true;

#[relib_module::export]
fn main() {
  // unsafe {
  //   USE_SYSTEM_ALLOC = false;
  // }
}
