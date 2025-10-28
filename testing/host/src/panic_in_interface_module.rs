use crate::shared::{ModuleExports, init_module_imports, load_module};

pub fn main() {
  let (module, _) = load_module::<ModuleExports, ()>(init_module_imports, true);
  unsafe {
    assert!(module.exports().panic().is_none());
  }
}
