mod allocator_proxy;

use {
  crate::update,
  allocator_proxy::ALLOC_PROXY,
  main_contract::{Alloc, Dealloc},
  relib_module as _,
  state::State,
  update_contract::Exports,
};

relib_interface::include_exports!(gen_exports, "update");
use gen_exports::ModuleExportsImpl;

impl Exports for ModuleExportsImpl {
  fn init_allocator_proxy(alloc: Alloc, dealloc: Dealloc) {
    ALLOC_PROXY.alloc.set(alloc).unwrap();
    ALLOC_PROXY.dealloc.set(dealloc).unwrap();
  }

  fn update(state: *mut ()) {
    let state = state_mut_ref(state);
    update(state);
  }
}

fn state_mut_ref<'a>(state: *mut ()) -> &'a mut State {
  // SAFETY: we logically receive state with exclusive access,
  // can't actually pass it as &mut because relib requires parameters
  // to be Copy to prevent module allocator touching foreign memory.
  // here it's safe because this module shares global allocator with
  // main module (see allocator_proxy.rs) and this state is only
  // mutated in this module.
  unsafe { &mut *(state as *mut State) }
}
