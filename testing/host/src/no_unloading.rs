use crate::shared::{ModuleExports, init_module_imports, load_module};

pub fn main() {
  let _ = load_module::<ModuleExports, ()>(init_module_imports, true);
}
