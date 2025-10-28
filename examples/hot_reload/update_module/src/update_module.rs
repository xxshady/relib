mod internal;

use state::State;

fn update(state: &mut State) {
  println!("---------- update ----------");

  state.counter += 1;

  // TODO: println has hidden state and it's leaked when this module is reloaded
  println!("update {}", state.counter);

  // demo of using library with only explicit state and ABI-stable types
  // from a perfect parallel universe:
  {
    let _entity = perfect_api::spawn(&mut state.perfect_api_state);
  } // despawned here

  // demo of using wrapper of library with hidden state and long compile times
  // (that's why it's implementation is in host binary) from real world:
  {
    let _entity = imperfect_api::spawn();
  } // also despawned here

  // all allocations are owned by main_module and
  // will not be deallocated when this module is unloaded
  // so, this is a temporary leak (until main_module is reloaded)
  state.vec.push(1);

  println!("---------- update ----------");
}
