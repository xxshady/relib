pub trait Exports {
  fn take_forget(vec: Vec<u8>);
  fn take_drop(vec: Vec<u8>);
  fn ret() -> Vec<u8>;
  fn nested_alloc(vec: Vec<Box<String>>);
}
