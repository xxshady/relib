use shared::exports2::Exports2;

relib_interface::include_exports!(gen_exports2, "module2");
relib_interface::include_imports!(gen_imports2, "module2");

impl Exports2 for gen_exports2::ModuleExportsImpl {
  fn bar2() -> u8 {
    30
  }
}

#[relib_module::export]
fn main() {
  println!("hello world2");

  let foo_value = unsafe { gen_imports2::foo2() };
  dbg!(foo_value);
}

// You can use it for joining threads or closing file handles and network sockets
// (if they are stored in static items for example)
#[cfg(feature = "unloading")]
#[relib_module::export]
fn before_unload() {
  println!("module2 is unloading...");
}
