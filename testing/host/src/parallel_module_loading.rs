use std::sync::atomic::{AtomicBool, Ordering::SeqCst};

use relib_host::LoadError;
use crate::shared::{self, init_module_imports, ModuleExports};

static SECOND_CALL: AtomicBool = AtomicBool::new(false);

pub fn main() {
  std::thread::scope(|s| {
    for _ in 1..=300 {
      s.spawn(move || {
        let res = shared::load_module_with_result::<ModuleExports, ()>(init_module_imports, true);

        let load_module_already_called = SECOND_CALL.load(SeqCst);
        match (res, load_module_already_called) {
          (Ok(_), false) => {
            SECOND_CALL.store(true, SeqCst);
          }
          (Err(LoadError::ModuleAlreadyLoaded), _) => {}
          (Ok(_), true) => {
            panic!("LoadError::ModuleAlreadyLoaded must be returned on second call to load_module");
          }
          (Err(e), _) => unreachable!("{e:?}"),
        }
      });
    }
  });
}
