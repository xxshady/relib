use {
  crate::startup,
  main_contract::{MainModuleRet, StableLayout},
  relib_module::AllocTracker,
  state::State,
  std::alloc::System,
};

// workaround for cargo feature unification (it should be registered by relib)
#[global_allocator]
static ALLOC_TRACKER: AllocTracker<System> = AllocTracker::new(System);

static mut STATE_PTR: *mut State = std::ptr::null_mut();

#[relib_module::export]
fn main() -> MainModuleRet {
  let state = startup();
  let state = Box::into_raw(Box::new(state));
  unsafe {
    STATE_PTR = state;
  }

  MainModuleRet {
    state: state as *mut (),
    alloc: alloc_tracker_alloc,
    dealloc: alloc_tracker_dealloc,
  }
}

#[relib_module::export]
fn before_unload() {
  unsafe {
    let state = Box::from_raw(STATE_PTR);
    drop(state);
  }
}

unsafe extern "C" fn alloc_tracker_alloc(layout: StableLayout) -> *mut u8 {
  unsafe { std::alloc::alloc(layout.into()) }
}

unsafe extern "C" fn alloc_tracker_dealloc(ptr: *mut u8, layout: StableLayout) {
  unsafe { std::alloc::dealloc(ptr, layout.into()) }
}
