pub mod exports;
pub mod imports;

pub const EXPORTS: &str = include_str!("exports.rs");
pub const IMPORTS: &str = include_str!("imports.rs");

pub fn build_id() -> u128 {
  env!("MAIN_CONTRACT_CRATE_BUILD_ID")
    .parse::<u128>()
    .unwrap()
}

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct StableLayout {
  pub size: usize,
  pub align: usize,
}

pub struct Kek {
  _state: state::State,
  // foo: u32,
}
