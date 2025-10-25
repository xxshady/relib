use {perfect_api::ApiState, state::State};

mod internal;

fn startup() -> State {
  let mut perfect_api_state = ApiState::default();

  // this module can also call libraries:
  {
    let _entity = perfect_api::spawn(&mut perfect_api_state);
  }
  {
    let _entity = unperfect_api::spawn();
  }

  State {
    counter: 0,
    perfect_api_state,

    vec: vec![1, 2, 3],
  }
}
