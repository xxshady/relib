use cfg_if::cfg_if;

use relib_host::{Module, ModuleExportsForHost};
use test_shared::{assert_mem_dealloc, print_memory_use, SIZE_200_MB};
use crate::shared::{
  init_module_imports, load_module, DropCallState, ModuleExports, THREAD_LOCAL_DROP_CALL_STATE,
};

fn unload_module<E: ModuleExportsForHost>(module: Module<E>) {
  cfg_if! {
    if #[cfg(feature = "unloading")] {
      module.unload().unwrap_or_else(|e| {
        panic!("{e:#}");
      });
    } else {
      drop(module);
      panic!("this branch must not be called");
    }
  }
}

pub fn main() {
  print_memory_use();

  // warm up the memory
  for _ in 1..=12 {
    unload_module(load_module::<(), ()>(init_module_imports).0);
    print_memory_use();
  }

  // TODO: check memory use here, on linux it should not be higher than 80MB
  print_memory_use();

  test_unloading_features();
}

fn test_unloading_features() {
  for _ in 1..=2 {
    println!("[host] loading module");

    assert_mem_dealloc(|| {
      let (module, _) = load_module::<ModuleExports, ()>(init_module_imports);

      unsafe {
        test_exports(module.exports()).unwrap();
      }

      println!("[host] unloading module");
      unload_module(module);
    });

    {
      let current = THREAD_LOCAL_DROP_CALL_STATE.get();
      assert_eq!(current, DropCallState::Called);
    }
  }
}

unsafe fn test_exports(exports: &ModuleExports) -> Option<()> {
  exports.empty()?;
  exports.empty_default()?;
  exports.dbg_default()?;

  let string = "1".repeat(100);
  let str = string.as_str().into();

  exports.ref_(str)?;
  let string_ = exports.ref_owned_ret(str)?;
  assert_eq!(string_, string);

  let ref_ = exports.ref_ret(str)?;
  assert_eq!(ref_, str[1..]);

  let ref_ = exports.ref_ret2(str, str)?;
  assert_eq!(ref_, str);

  exports.primitive(i32::MAX)?;

  let ret = exports.primitive_ret(i32::MIN)?;
  assert_eq!(ret, i32::MIN);

  THREAD_LOCAL_DROP_CALL_STATE.set(DropCallState::NotCalled);
  exports.thread_locals()?;

  assert_mem_dealloc(|| {
    let mem = exports.alloc_mem()?;
    assert_eq!(mem.len(), SIZE_200_MB);

    Some(())
  })?;

  exports.leak()?;

  exports.call_imports()?;

  // ------------------------------------------------------------------------
  let _suppress_unused_warn = || {
    #[expect(unreachable_code, clippy::diverging_sub_expression)]
    exports._params_lt_and_output_without(unreachable!(), unreachable!())?;

    Some(())
  };

  Some(())
}
