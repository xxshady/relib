use abi_stable::std_types::{RStr, RString, RVec};

pub trait Exports {
  // ------------------------------- codegen testing

  fn empty();
  fn empty_default() {}

  fn dbg_default() {
    dbg!();
  }

  fn ref_(r: RStr);

  #[expect(clippy::needless_lifetimes)]
  fn ref_ret<'a>(r: RStr<'a>) -> RStr<'a>;
  fn ref_ret2<'a>(r: RStr<'a>, r2: RStr<'a>) -> RStr<'a>;

  fn _params_lt_and_output_without<'a>(r: RStr<'a>, r2: RStr<'a>) -> RString;

  fn ref_owned_ret(r: RStr) -> RString;

  fn primitive(p: i32);
  fn primitive_ret(p: i32) -> i32;

  fn alloc_mem() -> RVec<u8>;

  fn panic();
  fn call_host_panic();

  // ------------------------------- other

  fn leak();
  fn thread_locals();

  fn call_imports();

  fn only_called_once() -> bool;
}
