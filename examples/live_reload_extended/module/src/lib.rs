use shared::exports::Exports;

relib_interface::include_exports!();
use gen_exports::ModuleExportsImpl;
relib_interface::include_imports!();

impl Exports for ModuleExportsImpl {
  fn shared_build_id() -> u128 {
    shared::build_id()
  }
}

#[relib_module::export]
fn main() {
  let foo_ = unsafe { gen_imports::foo() };
  println!("change me! {foo_} but don't touch shared crate");
}
