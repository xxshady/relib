use {
  crate::unloading_core::helpers::unrecoverable,
  hashbrown::{DefaultHashBuilder, HashMap},
  relib_internal_shared::AllocatorPtr,
  std::{
    alloc::{GlobalAlloc, Layout, System},
    sync::{LazyLock, Mutex},
  },
};

#[global_allocator]
static ALLOC: AllocLayoutTracker<System> = AllocLayoutTracker { allocator: System };

pub struct AllocLayoutTracker<G>
where
  G: GlobalAlloc,
{
  allocator: G,
}

unsafe impl<G> GlobalAlloc for AllocLayoutTracker<G>
where
  G: GlobalAlloc,
{
  unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
    // SAFETY: TODO:
    let ptr = unsafe { self.allocator.alloc(layout) };

    ALLOC_LAYOUTS
      .lock()
      .unwrap_or_else(|_| {
        unrecoverable("global alloc layouts lock failed");
      })
      .insert(AllocatorPtr(ptr), layout);

    ptr
  }

  unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
    let removed_layout = ALLOC_LAYOUTS
      .lock()
      .unwrap_or_else(|_| {
        unrecoverable("global dealloc layouts lock failed");
      })
      .remove(&AllocatorPtr(ptr));

    // TODO: also add this check to module dealloc_validation
    // TODO: add test for it
    if removed_layout != Some(layout) {
      unrecoverable("global dealloc was called with invalid ptr + layout combination");
    }

    // SAFETY: TODO:
    unsafe {
      self.allocator.dealloc(ptr, layout);
    }
  }
}

// TODO: make it possible to replace allocator for this hashmap for relib users
static ALLOC_LAYOUTS: LazyLock<Mutex<HashMap<AllocatorPtr, Layout, DefaultHashBuilder, System>>> =
  LazyLock::new(Default::default);

pub fn layout_of(ptr: *mut u8) -> Layout {
  *ALLOC_LAYOUTS
    .lock()
    .unwrap_or_else(|_| {
      unrecoverable("global alloc layout store: lock failed");
    })
    .get(&AllocatorPtr(ptr))
    .unwrap_or_else(|| {
      unrecoverable("global alloc layout store: invalid ptr");
    })
}
