relib_interface::include_exports!(gen_exports, "update");
use anyhow::anyhow;
use gen_exports::ModuleExports;

relib_interface::include_imports!(gen_imports, "update");
use gen_imports::{init_imports, ModuleImportsImpl};
use main_contract::{StableLayout, imports::Imports};
use relib_host::Module;
use state::State;

use crate::{
  CALL_MAIN_MODULE_ALLOC, CALL_MAIN_MODULE_DEALLOC, MainModuleImportsImpl,
  shared::{AnyErrorResult, BuildResult, build_module, load_module},
};

impl Imports for ModuleImportsImpl {
  fn foo() -> i32 {
    MainModuleImportsImpl::foo()
  }

  fn alloc(layout: StableLayout) -> *mut u8 {
    CALL_MAIN_MODULE_ALLOC.with_borrow(|f| f(layout))
  }

  fn dealloc(ptr: *mut u8, layout: StableLayout) {
    CALL_MAIN_MODULE_DEALLOC.with_borrow(|f| f(ptr, layout))
  }
}

pub struct UpdateModule {
  module: Option<Module<ModuleExports>>,
  build_failed_in_prev_iteration: bool,
}

impl UpdateModule {
  pub fn load() -> AnyErrorResult<UpdateModule> {
    let module = Self::load_()?;
    Ok(Self {
      module: Some(module),
      build_failed_in_prev_iteration: false,
    })
  }

  fn load_() -> AnyErrorResult<Module<ModuleExports>> {
    let module = load_module("update", init_imports)?;
    unsafe {
      () = module.call_main().unwrap();
    }

    Ok(module)
  }

  pub fn rebuild(&mut self) -> AnyErrorResult<()> {
    let build_res = build_module("update")?;
    match build_res {
      BuildResult::Success => {
        dbg!();
        // when unloading fails it is not safe to load it again
        self
          .module
          .take()
          .unwrap()
          .unload()
          .map_err(|e| anyhow!("update module unloading failed: {e:#}"))?;
        dbg!();

        println!("update module has been rebuilt");

        self.module = Some(Self::load_()?);
        self.build_failed_in_prev_iteration = false;
      }
      BuildResult::Failure(message) => {
        if self.build_failed_in_prev_iteration {
          return Ok(());
        }
        self.build_failed_in_prev_iteration = true;

        println!("failed to build the update module:\n{message}");
      }
      BuildResult::NoChange => {}
    }

    Ok(())
  }

  pub unsafe fn update(&self, state: *mut State) {
    unsafe {
      self
        .module
        .as_ref()
        .unwrap()
        .exports()
        .update(state)
        .unwrap()
    }
  }
}
