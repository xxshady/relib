// workaround to disable "global_alloc_tracker" feature
// (it can't be disabled because it's in default features of relib_module crate)
// and to not break `cargo build --workspace`
#[cfg(feature = "custom_global_alloc")]
mod module;
