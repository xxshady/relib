mod allocator_proxy;

use main_contract::{Alloc, Dealloc};
use relib_module as _;

use state::State;

use update_contract::exports::Exports;

relib_interface::include_exports!(gen_exports, "update");
use gen_exports::ModuleExportsImpl;

use crate::{allocator_proxy::ALLOC_PROXY};
relib_interface::include_imports!(gen_imports, "update");

impl Exports for ModuleExportsImpl {
  fn init_allocator_proxy(alloc: Alloc, dealloc: Dealloc) {
    ALLOC_PROXY.alloc.set(alloc).unwrap();
    ALLOC_PROXY.dealloc.set(dealloc).unwrap();
  }

  fn update(state: *mut ()) {
    let state = state_mut_ref(state);

    state.counter += 1;

    // TODO: println has hidden state so it leaks when this module is reloaded
    println!("update {}", state.counter);

    // demo of using library with only explicit state and ABI-stable types
    // from a perfect parallel universe:
    {
      let _entity = perfect_api::spawn(&mut state.api_state);
    } // despawned here

    // demo of using library with hidden state from real world:
    unsafe {
      let entity = gen_imports::spawn_entity_from_not_perfect_parallel_universe();

      gen_imports::despawn_entity_from_not_perfect_parallel_universe(entity);
    }

    // all allocations are owned by main_module and
    // will not be deallocated when this module is unloaded
    // so, this is a temporary leak (until main_module is reloaded)
    // TODO: rename module crate to main_module
    // TODO: or even better: state_module?
    // TODO: and then this module should also be renamed if it will contain startup
    // for _ in 0..(1024 * 1024) {
    //   state.vec.push(1);
    // }
  }
}

#[relib_module::export]
fn before_unload() {
  println!("[update] before unload");
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
