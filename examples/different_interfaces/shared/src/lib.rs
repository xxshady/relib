pub mod exports;
pub mod imports;

pub const EXPORTS: &str = include_str!("exports.rs");
pub const IMPORTS: &str = include_str!("imports.rs");

pub mod exports2;
pub mod imports2;

pub const EXPORTS2: &str = include_str!("exports2.rs");
pub const IMPORTS2: &str = include_str!("imports2.rs");
