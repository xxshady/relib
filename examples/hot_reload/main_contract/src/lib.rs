pub mod exports;
pub mod shared_imports;

pub const EXPORTS: &str = include_str!("exports.rs");
pub const SHARED_IMPORTS: &str = include_str!("shared_imports.rs");

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct StableLayout {
  pub size: usize,
  pub align: usize,
}
