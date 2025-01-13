#[relib_module::export]
pub fn main() {}

#[relib_module::export]
pub fn before_unload() {
  panic!("expected panic");
}
