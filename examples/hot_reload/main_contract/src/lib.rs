pub mod exports;
pub mod imports;

pub const EXPORTS: &str = include_str!("exports.rs");
pub const IMPORTS: &str = include_str!("imports.rs");

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct StableLayout {
  pub size: usize,
  pub align: usize,
}
