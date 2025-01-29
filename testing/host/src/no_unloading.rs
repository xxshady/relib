use crate::shared::{init_module_imports, load_module, ModuleExports};

pub fn main() {
  let _ = load_module::<ModuleExports, ()>(init_module_imports, true);
}
