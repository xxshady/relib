use state::State;

pub trait Exports {
  fn update(state: *mut State);
}
