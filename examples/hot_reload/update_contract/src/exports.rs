pub trait Exports {
  fn main_contract_build_id() -> u128;
  fn update(state: *mut ());
}
