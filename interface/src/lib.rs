#[cfg(feature = "build")]
pub mod host;
#[cfg(feature = "build")]
pub mod module;
#[cfg(feature = "build")]
mod shared;

#[cfg(feature = "include")]
#[macro_export]
macro_rules! include_exports {
  () => {
    $crate::include_exports!(gen_exports);
  };
  ($mod_name:ident) => {
    mod $mod_name {
      include!(concat!(env!("OUT_DIR"), "/generated_module_exports.rs"));
    }
  };
}

#[cfg(feature = "include")]
#[macro_export]
macro_rules! include_imports {
  () => {
    $crate::include_imports!(gen_imports);
  };
  ($mod_name:ident) => {
    mod $mod_name {
      include!(concat!(env!("OUT_DIR"), "/generated_module_imports.rs"));
    }
  };
}
