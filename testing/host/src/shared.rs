use relib_host::{InitImports, Module, ModuleExportsForHost};

pub fn load<T: ModuleExportsForHost>(init_imports: impl InitImports) -> Module<T> {
  let directory = if cfg!(debug_assertions) {
    "debug"
  } else {
    "release"
  };

  let path = if cfg!(target_os = "linux") {
    format!("target/{directory}/libtest_module.so")
  } else {
    format!("target/{directory}/test_module.dll")
  };

  let module = relib_host::load_module::<T>(path, init_imports).unwrap_or_else(|e| {
    panic!("{e:#}");
  });

  unsafe {
    module.call_main::<()>().unwrap();
  }
  module
}
