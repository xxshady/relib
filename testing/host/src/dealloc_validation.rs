use crate::shared::{init_module_imports, load_module};

pub fn main() {
  eprintln!("this is expected:");
  let (_, _) = load_module::<(), ()>(init_module_imports, true);
}
