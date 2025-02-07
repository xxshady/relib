use crate::shared::{init_module_imports, load_module, ModuleExports};

pub fn main() {
  let (module, _) = load_module::<ModuleExports, ()>(init_module_imports, true);
  unsafe {
    module.exports().call_host_panic().unwrap();
  }
}
