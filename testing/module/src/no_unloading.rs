relib_interface::include_imports!();
relib_interface::include_exports!();

pub fn main() {
  unsafe {
    dbg!(
      // gen_imports::b(),
      dbg!(gen_imports::with_return_value(i32::MIN)) == i32::MIN,
      dbg!(gen_imports::with_return_value(i32::MAX)) == i32::MAX
    );
  }
}
