use {
  crate::startup,
  main_contract::{Exports, MainModuleRet, StableLayout},
  relib_module::AllocTracker,
  state::State,
  std::alloc::System,
};

relib_interface::include_exports!();
use gen_exports::ModuleExportsImpl;

impl Exports for ModuleExportsImpl {
  fn drop_state(state: *mut ()) {
    println!("---------- state drop ----------");
    unsafe {
      let state = Box::from_raw(state as *mut State);
      drop(state);
    }
    println!("---------- state drop ----------");
  }
}

// workaround for cargo feature unification (it should be registered by relib)
#[global_allocator]
static ALLOC_TRACKER: AllocTracker<System> = AllocTracker::new(System);

#[relib_module::export]
fn main() -> MainModuleRet {
  let state = startup();
  let state = Box::into_raw(Box::new(state));

  MainModuleRet {
    state: state as *mut (),
    alloc: alloc_tracker_alloc,
    dealloc: alloc_tracker_dealloc,
  }
}

unsafe extern "C" fn alloc_tracker_alloc(layout: StableLayout) -> *mut u8 {
  unsafe { std::alloc::alloc(layout.into()) }
}

unsafe extern "C" fn alloc_tracker_dealloc(ptr: *mut u8, layout: StableLayout) {
  unsafe { std::alloc::dealloc(ptr, layout.into()) }
}
