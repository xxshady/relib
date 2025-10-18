use std::alloc::{GlobalAlloc, Layout, System};
use main_contract::StableLayout;
use relib_module::AllocTracker;
use crate::gen_imports;

struct Proxy;

unsafe impl GlobalAlloc for Proxy {
  unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
    unsafe {
      if USE_SYSTEM_ALLOC {
        return System.alloc(layout);
      }

      gen_imports::alloc(StableLayout {
        size: layout.size(),
        align: layout.align(),
      })
    }
  }

  unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
    unsafe {
      if !DEALLOC_ALLOWED {
        return;
      }

      if USE_SYSTEM_ALLOC {
        return System.dealloc(ptr, layout);
      }

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
static PROXY: AllocTracker<Proxy> = AllocTracker::new(Proxy);

// TODO: fix this properly
static mut USE_SYSTEM_ALLOC: bool = true;

// disabling "leaks deallocation" because it tries to deallocate
// memory of State struct which is owned by main module
static mut DEALLOC_ALLOWED: bool = true;

#[relib_module::export]
fn main() {
  unsafe {
    USE_SYSTEM_ALLOC = false;
  }
}

#[relib_module::export]
fn before_unload() {
  unsafe {
    DEALLOC_ALLOWED = false;
  }
}
