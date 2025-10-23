use perfect_api::ApiState;

#[repr(C)]
#[derive(Debug)]
pub struct State {
  pub counter: u32,
  pub api_state: ApiState,

  // TODO: should not be used here since it's not ABI-stable
  pub vec: Vec<u8>,
}
