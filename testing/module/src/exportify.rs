use cfg_if::cfg_if;

cfg_if! {
  if #[cfg(feature = "ret_primitive_main")] {
    #[relib_module::export]
    pub fn main() -> u64 {
      println!("[module] ret_primitive_main");
      u64::MAX
    }
  } else if #[cfg(feature = "ret_heap_main")] {
    use abi_stable::std_types::RString;

    #[relib_module::export]
    pub fn main() -> RString {
      println!("[module] ret_heap_main");
      "ret_heap_main".into()
    }
  } else if #[cfg(feature = "panic_main")] {
    #[relib_module::export]
    pub fn main() {
      println!("[module] panic_main");
      panic!("expected panic");
    }
  } else {
    #[relib_module::export]
    pub fn main() {
      println!("[module] default main");
    }
  }
}
