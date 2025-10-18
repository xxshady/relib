use shared::exports::Exports;

relib_interface::include_exports!();
relib_interface::include_imports!();

impl Exports for gen_exports::ModuleExportsImpl {
  fn bar() -> u8 {
    15
  }
}

#[relib_module::export]
fn main() {
  println!("hello world");

  let foo_value = unsafe { gen_imports::foo() };
  dbg!(foo_value);
}

// You can use it for joining threads or closing file handles and network sockets
// (if they are stored in static items for example)
#[cfg(feature = "unloading")]
#[relib_module::export]
fn before_unload() {
  println!("module is unloading...");
}
