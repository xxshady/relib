use std::cell::Cell;

use abi_stable::std_types::{RStr, RString, RVec};
use relib_host::{InitImports, Module, ModuleExportsForHost};
use test_shared::{imports::Imports, SIZE_200_MB};

relib_interface::include_exports!();
relib_interface::include_imports!();

pub use gen_exports::ModuleExports;
pub use gen_imports::{init_imports as init_module_imports, ModuleImportsImpl};

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

pub fn load_module<Exports: ModuleExportsForHost, MainRet: Clone>(
  init_imports: impl InitImports,
  check_panic: bool,
) -> (Module<Exports>, Option<MainRet>) {
  load_module_with_name(init_imports, "test_module", check_panic)
}

pub fn load_module_with_name<Exports: ModuleExportsForHost, MainRet: Clone>(
  init_imports: impl InitImports,
  name: &str,
  check_panic: bool,
) -> (Module<Exports>, Option<MainRet>) {
  let directory = if cfg!(debug_assertions) {
    "debug"
  } else {
    "release"
  };

  let path = if cfg!(target_os = "linux") {
    format!("target/{directory}/lib{name}.so")
  } else {
    format!("target/{directory}/{name}.dll")
  };

  let module = relib_host::load_module::<Exports>(path, init_imports).unwrap_or_else(|e| {
    panic!("{e:#}");
  });

  let ret = unsafe { module.call_main::<MainRet>() };

  if check_panic {
    assert!(ret.is_some(), "module main fn panicked");
  }

  (module, ret)
}
