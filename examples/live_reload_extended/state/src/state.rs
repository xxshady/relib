#[repr(C)]
#[derive(Debug)]
pub struct State {
  pub foo: u32,

  // should be ABI stable but it's fine for prototype
  pub bar: Vec<u8>,

  pub baz: u8,
}
