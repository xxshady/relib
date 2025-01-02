use std::{
  alloc::{GlobalAlloc, Layout, System},
  backtrace::Backtrace,
  hint::black_box,
  mem::forget,
  sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering::Relaxed},
    Mutex,
  },
};

struct Counter;

pub static ALLOCS: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for Counter {
  unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
    let ret = System.alloc(layout);
    if IGNORE.load(Relaxed) {
      return ret;
    }

    if !ret.is_null() {
      ALLOCS.fetch_add(1, Relaxed);
    }
    ret
  }

  unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
    System.dealloc(ptr, layout);
    if IGNORE.load(Relaxed) {
      return;
    }

    if STORE_TRACES.load(Relaxed) {
      IGNORE.store(true, Relaxed);
      {
        let mut traces = DEALLOC_TRACES.lock().unwrap();
        traces.push(Backtrace::force_capture());
      }
      IGNORE.store(false, Relaxed);
    }

    ALLOCS.fetch_sub(1, Relaxed);
  }
}

#[global_allocator]
static ALLOC_COUNTER: Counter = Counter;

pub static IGNORE: AtomicBool = AtomicBool::new(false);
pub static STORE_TRACES: AtomicBool = AtomicBool::new(false);

pub static DEALLOC_TRACES: Mutex<Vec<Backtrace>> = Mutex::new(Vec::new());
