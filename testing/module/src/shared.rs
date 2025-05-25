use std::{
  mem::forget,
  sync::{
    atomic::{AtomicBool, Ordering::Relaxed},
    Mutex, MutexGuard,
  },
  thread::{self, JoinHandle},
};

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
        // TODO: add custom println for testing? since println uses thread-local
        // println!("[module] thread local drop called");

        unsafe {
          gen_imports::thread_local_drop_called();
        }
      }
    }

    thread_local! {
      static TLS_WITH_DROP: TlsWithDrop = TlsWithDrop {
        _mem: alloc_some_bytes(),
      };

      static TLS_WITH_DROP2: TlsWithDrop2 = TlsWithDrop2 {
        _mem: alloc_some_bytes(),
      };

      static TLS_WITH_DROP3: TlsWithDrop2 = TlsWithDrop2 {
        _mem: alloc_some_bytes(),
      };
    }

    println!("before init of thread local");
    TLS_WITH_DROP.with(|v| {
      println!("after init of thread local");
      assert_eq!(v._mem.len(), SIZE_200_MB);
    });

    struct TlsWithDrop2 {
      _mem: Vec<u8>,
    }

    println!("22222222 before init of thread local");
    TLS_WITH_DROP2.with(|v| {
      println!("22222222 after init of thread local");
      assert_eq!(v._mem.len(), SIZE_200_MB);
    });

    println!("33333333 before init of thread local");
    TLS_WITH_DROP3.with(|v| {
      println!("33333333 after init of thread local");
      assert_eq!(v._mem.len(), SIZE_200_MB);
    });

    struct AnotherTlsWithDrop {
      _mem: Vec<u8>,
    }

    static DROP_IN_THREAD_CALLED: AtomicBool = AtomicBool::new(false);
    impl Drop for AnotherTlsWithDrop {
      fn drop(&mut self) {
        // TODO: add custom println for testing? since println uses thread-local
        // println!("[module] thread local 2 drop called");

        DROP_IN_THREAD_CALLED.store(true, Relaxed);
      }
    }

    thread_local! {
      static ANOTHER_TLS_WITH_DROP: AnotherTlsWithDrop = AnotherTlsWithDrop {
        _mem: alloc_some_bytes(),
      }
    }

    thread::spawn(|| {
      ANOTHER_TLS_WITH_DROP.with(|_| {});
    })
    .join()
    .unwrap();

    assert!(DROP_IN_THREAD_CALLED.load(Relaxed));
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

  fn only_called_once() -> bool {
    static mut CALLED: bool = false;
    unsafe {
      let val = CALLED;
      CALLED = true;
      !val
    }
  }

  fn spawn_background_threads(module_id: u64, how_many: u8) {
    for i in 0..how_many {
      let handle = thread::spawn(move || {
        thread::park();
        println!("[module: {module_id}] thread {i} unparked");
      });
      background_threads().push(handle);
    }
  }

  fn join_background_threads() {
    background_threads().drain(..).for_each(|handle| {
      handle.thread().unpark();
      handle.join().unwrap();
    });
  }
}

pub fn alloc_some_bytes() -> Vec<u8> {
  vec![1_u8; SIZE_200_MB]
}

fn background_threads() -> MutexGuard<'static, Vec<JoinHandle<()>>> {
  static HANDLES: Mutex<Vec<JoinHandle<()>>> = Mutex::new(Vec::new());

  HANDLES.lock().unwrap()
}
