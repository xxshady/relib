use {
  abi_stable::std_types::{RStr, RString, RVec},
  std::cell::Cell,
  test_shared::{SIZE_200_MB, imports::Imports},
};
pub use test_host_shared::*;

relib_interface::include_exports!();
relib_interface::include_imports!();

pub use {
  gen_exports::ModuleExports,
  gen_imports::{ModuleImportsImpl, init_imports as init_module_imports},
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DropCallState {
  None,
  NotCalled,
  Called,
}

thread_local! {
  pub static THREAD_LOCAL_DROP_CALL_STATE: Cell<DropCallState> = const { Cell::new(DropCallState::None) };
}

impl Imports for ModuleImportsImpl {
  // ------------------------------- codegen testing

  fn empty() {}

  fn ref_(r: RStr) {
    assert_eq!(r, "1".repeat(100));
  }

  fn ref_owned_ret(r: RStr) -> RString {
    r.into()
  }

  fn ref_ret(str: RStr) -> RStr {
    str.slice(1..)
  }

  fn ref_ret2<'a>(r: RStr<'a>, _r2: RStr<'a>) -> RStr<'a> {
    r
  }

  fn _params_lt_and_output_without<'a>(_: RStr<'a>, _: RStr<'a>) -> RString {
    unreachable!()
  }

  fn primitive(p: i32) {
    assert_eq!(p, i32::MAX);
  }

  fn primitive_ret(p: i32) -> i32 {
    assert_eq!(p, i32::MIN);
    p
  }

  fn alloc_mem() -> RVec<u8> {
    alloc_some_bytes().into()
  }

  fn panic() {
    panic!("expected panic");
  }

  // ------------------------------- other

  fn thread_local_drop_called() {
    let current = THREAD_LOCAL_DROP_CALL_STATE.get();
    assert_eq!(current, DropCallState::NotCalled);
    THREAD_LOCAL_DROP_CALL_STATE.set(DropCallState::Called);
  }
}

fn alloc_some_bytes() -> Vec<u8> {
  vec![1_u8; SIZE_200_MB]
}
