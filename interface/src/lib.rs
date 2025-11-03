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
    $crate::include_exports!(gen_exports, "generated_module");
  };
  ($mod_name:ident) => {
    $crate::include_exports!($mod_name, "generated_module");
  };
  ($mod_name:ident, $prefix:literal) => {
    mod $mod_name {
      include!(concat!(
        env!("OUT_DIR"),
        env!(concat!("__RELIB_OUT_DIR_", $prefix, "_exports__"))
      ));
    }
  };
}

// TODO: use it in examples
/// Shortcut for:
/// ```
/// relib_interface::include_exports!();
/// relib_interface::include_imports!();
/// ```
#[cfg(feature = "include")]
#[macro_export]
macro_rules! include_all {
  () => {
    relib_interface::include_exports!();
    relib_interface::include_imports!();
  };
}

#[cfg(feature = "include")]
#[macro_export]
macro_rules! include_imports {
  () => {
    $crate::include_imports!(gen_imports, "generated_module");
  };
  ($mod_name:ident) => {
    $crate::include_imports!($mod_name, "generated_module");
  };
  ($mod_name:ident, $prefix:literal) => {
    mod $mod_name {
      include!(concat!(
        env!("OUT_DIR"),
        env!(concat!("__RELIB_OUT_DIR_", $prefix, "_imports__"))
      ));
    }
  };
}
