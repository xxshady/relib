use {perfect_api::ApiState, state::State};

mod internal;

fn startup() -> State {
  println!("---------- startup ----------");

  let mut perfect_api_state = ApiState::default();

  // this module can also call libraries:

  println!("spawning entity that will be despawned by state drop:");
  let entity = perfect_api::spawn(&mut perfect_api_state);

  println!("spawning leaked entity:");

  let entity_leak = imperfect_api::spawn();
  // host will despawn it for us at reload of this module
  std::mem::forget(entity_leak);

  {
    let _entity = imperfect_api::spawn();
  }

  let state = State {
    counter: 0,
    perfect_api_state,
    entity,

    vec: vec![1, 2, 3].into(),
  };

  println!("---------- startup ----------");

  state
}
