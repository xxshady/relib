use std::env::consts::{DLL_PREFIX, DLL_SUFFIX};

use libloading::Library;

use crate::shared::current_target_dir;

pub fn main() {
  let target_dir = current_target_dir();
  let path = format!("{target_dir}/backtrace_unloading_host_as_dylib__host/{DLL_PREFIX}test_host_as_dylib{DLL_SUFFIX}");

  unsafe {
    let host = Library::new(path).unwrap();
    let symbol = host.get(b"main\0").unwrap();
    let main: extern "C" fn() = *symbol;
    main();
  }
}
