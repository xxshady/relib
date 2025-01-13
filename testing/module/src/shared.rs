use std::{mem::forget, thread};

use abi_stable::std_types::{RStr, RString, RVec};

use test_shared::{assert_mem_dealloc, exports::Exports, SIZE_200_MB};

relib_interface::include_imports!();
relib_interface::include_exports!();
use gen_exports::ModuleExportsImpl;

impl Exports for ModuleExportsImpl {
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

  fn call_imports() {
    unsafe {
      gen_imports::empty();
      gen_imports::empty_default();
      gen_imports::dbg_default();

      let string = "1".repeat(100);
      let str = string.as_str().into();

      gen_imports::ref_(str);
      let string_ = gen_imports::ref_owned_ret(str);
      assert_eq!(string_, string);

      let ref_ = gen_imports::ref_ret(str);
      assert_eq!(ref_, str[1..]);

      let ref_ = gen_imports::ref_ret2(str, str);
      assert_eq!(ref_, str);

      gen_imports::primitive(i32::MAX);

      let ret = gen_imports::primitive_ret(i32::MIN);
      assert_eq!(ret, i32::MIN);

      assert_mem_dealloc(|| {
        let mem = gen_imports::alloc_mem();
        assert_eq!(mem.len(), SIZE_200_MB);
      });

      // ------------------------------------------------------------------------
      let _suppress_unused_warn = || {
        #[expect(unreachable_code, clippy::diverging_sub_expression)]
        gen_imports::_params_lt_and_output_without(unreachable!(), unreachable!());
      };
    }
  }

  fn leak() {
    forget(alloc_some_bytes());
  }

  fn thread_locals() {
    struct TlsWithDrop {
      _mem: Vec<u8>,
    }

    impl Drop for TlsWithDrop {
      fn drop(&mut self) {
        println!("[module] thread local drop called");
        unsafe {
          gen_imports::thread_local_drop_called();
        }
      }
    }

    thread_local! {
      static TLS_WITH_DROP: TlsWithDrop = TlsWithDrop {
        _mem: alloc_some_bytes(),
      }
    }

    TLS_WITH_DROP.with(|_| {});

    struct TlsWithDrop2 {
      _mem: Vec<u8>,
    }

    impl Drop for TlsWithDrop2 {
      fn drop(&mut self) {
        println!("[module] thread local 2 drop called");
      }
    }

    thread_local! {
      static TLS_WITH_DROP2: TlsWithDrop2 = TlsWithDrop2 {
        _mem: alloc_some_bytes(),
      }
    }

    thread::spawn(|| {
      TLS_WITH_DROP2.with(|_| {});
    })
    .join()
    .unwrap();
  }

  fn alloc_mem() -> RVec<u8> {
    alloc_some_bytes().into()
  }

  fn panic() {
    panic!("expected panic");
  }

  fn call_host_panic() {
    unsafe {
      gen_imports::panic();
    }
  }
}

pub fn alloc_some_bytes() -> Vec<u8> {
  vec![1_u8; SIZE_200_MB]
}
