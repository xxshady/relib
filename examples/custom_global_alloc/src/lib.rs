use std::{
  alloc::{GlobalAlloc, Layout, System},
  sync::atomic::{AtomicUsize, Ordering::Relaxed},
};

use relib_module::AllocTracker;

struct Counter;

static ALLOCATED: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for Counter {
  unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
    let ret = System.alloc(layout);
    if !ret.is_null() {
      ALLOCATED.fetch_add(layout.size(), Relaxed);
    }
    ret
  }

  unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
    System.dealloc(ptr, layout);
    ALLOCATED.fetch_sub(layout.size(), Relaxed);
  }
}

#[global_allocator]
static ALLOC_COUNTER: AllocTracker<Counter> = AllocTracker::new(Counter);

#[relib_module::export]
fn main() {
  println!("allocated bytes before main: {}", ALLOCATED.load(Relaxed));
}
