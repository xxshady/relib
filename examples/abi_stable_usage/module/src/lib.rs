use abi_stable::std_types::RString;
use shared::exports::Exports;

relib_interface::include_exports!();
use gen_exports::ModuleExportsImpl;

relib_interface::include_imports!();

impl Exports for ModuleExportsImpl {
  fn bar() -> RString {
    "FFI-safe string!".into()
  }
}

#[relib_module::export]
fn main() {
  println!("hello world");

  let bar_value = unsafe { gen_imports::foo() };
  dbg!(bar_value);
}
