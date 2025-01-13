use cfg_if::cfg_if;

use crate::shared::{init_module_imports, load_module};

cfg_if! {
  if #[cfg(feature = "ret_primitive_main")] {
    type MainRet = u64;

    fn check_main_ret(ret: MainRet) {
      assert_eq!(ret, u64::MAX);
    }
  } else if #[cfg(feature = "ret_heap_main")] {
    use abi_stable::std_types::RString;

    type MainRet = RString;

    fn check_main_ret(ret: RString) {
      assert_eq!(ret, "ret_heap_main");
    }
  } else if #[cfg(feature = "panic_main")] {
    type MainRet = ();

    fn check_main_ret(_ret: ()) {}
  } else {
    type MainRet = ();

    fn check_main_ret(_ret: ()) {}
  }
}

pub fn main() {
  let (_, ret) = load_module::<(), MainRet>(init_module_imports);

  if cfg!(feature = "panic_main") {
    assert_eq!(ret, None);
    println!("expected panic, everything is ok");
    return;
  }

  let ret = ret.unwrap();
  check_main_ret(ret);
}
