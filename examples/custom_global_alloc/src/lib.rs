use std::{
  alloc::{GlobalAlloc, Layout, System},
  hint::black_box,
  mem::forget,
  sync::atomic::{AtomicUsize, Ordering::Relaxed},
};

struct Counter;

static ALLOCS: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for Counter {
  unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
    let ret = unsafe { System.alloc(layout) };
    if !ret.is_null() {
      ALLOCS.fetch_add(1, Relaxed);
    }
    ret
  }

  unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
    unsafe {
      System.dealloc(ptr, layout);
    }
    ALLOCS.fetch_sub(1, Relaxed);
  }
}

#[cfg(feature = "unloading")]
mod with_unloading {
  use {super::Counter, relib_module::AllocTracker};

  #[global_allocator]
  static ALLOC_COUNTER: AllocTracker<Counter> = AllocTracker::new(Counter);
}

#[cfg(not(feature = "unloading"))]
#[global_allocator]
static ALLOC_COUNTER: Counter = Counter;

#[relib_module::export]
fn main() {
  let before = ALLOCS.load(Relaxed);
  forget(black_box(vec![1_u8]));
  let after = ALLOCS.load(Relaxed);

  println!("allocations before: {before}");
  println!("allocations after: {after}");
}
