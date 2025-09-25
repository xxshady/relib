pub mod exports;
pub mod imports;

pub const EXPORTS: &str = include_str!("exports.rs");
pub const IMPORTS: &str = include_str!("imports.rs");

pub fn build_id() -> u128 {
  env!("SHARED_CRATE_BUILD_ID").parse::<u128>().unwrap()
}
