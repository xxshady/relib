use std::alloc::System;
use main_contract::{MainModuleRet, StableLayout};
use perfect_api::ApiState;
use state::State;
use relib_module::AllocTracker;

relib_interface::include_imports!();

// workaround for cargo feature unification (it should be registered by relib)
#[global_allocator]
static ALLOC_TRACKER: AllocTracker<System> = AllocTracker::new(System);

#[relib_module::export]
unsafe fn main() -> MainModuleRet {
  let state = State {
    counter: 0,
    api_state: ApiState::default(),

    vec: vec![1, 2, 3],
  };

  // TODO: should this module use external libraries?
  // unsafe {
  //   let entity = gen_imports::spawn_entity_from_not_perfect_parallel_universe();
  //   gen_imports::despawn_entity_from_not_perfect_parallel_universe(entity);
  // }

  MainModuleRet {
    state: Box::into_raw(Box::new(state)) as *mut (),
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
