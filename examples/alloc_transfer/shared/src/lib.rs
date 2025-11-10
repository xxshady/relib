mod exports;
mod imports;

pub use {exports::Exports, imports::Imports};

pub const EXPORTS: &str = include_str!("exports.rs");
pub const IMPORTS: &str = include_str!("imports.rs");
