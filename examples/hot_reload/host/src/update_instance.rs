relib_interface::include_exports!(gen_exports, "update");
use anyhow::anyhow;
use gen_exports::ModuleExports;

relib_interface::include_imports!(gen_imports, "update");
use gen_imports::{init_imports, ModuleImportsImpl};
use main_contract::{StableLayout, imports::Imports};
use relib_host::Module;

use crate::{
  CALL_MAIN_MODULE_ALLOC, CALL_MAIN_MODULE_DEALLOC, MainModuleImportsImpl,
  shared::{AnyErrorResult, load_module},
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
}

impl UpdateModule {
  pub fn load() -> AnyErrorResult<UpdateModule> {
    let module = Self::load_()?;
    Ok(Self {
      module: Some(module),
    })
  }

  fn load_() -> AnyErrorResult<Module<ModuleExports>> {
    let module: Module<ModuleExports> = load_module("update", init_imports, false)?;

    let module_main_contract_build_id =
      unsafe { module.exports().main_contract_build_id().unwrap() };
    let host_main_contract_build_id = main_contract::build_id();

    // when main_contract crate is modified it's no longer safe to load the module,
    // so we need to stop here
    if module_main_contract_build_id != host_main_contract_build_id {
      return Err(anyhow!(
        "main_contract crate was modified, update module potentially contains incompatible code\n\
        main_contract build id of:\n\
        host:   {}\n\
        module: {}",
        host_main_contract_build_id,
        module_main_contract_build_id
      ));
    }

    Ok(module)
  }

  pub fn reload(&mut self) -> AnyErrorResult<()> {
    // when unloading fails it is not safe to load it again
    self
      .module
      .take()
      .unwrap()
      .unload()
      .map_err(|e| anyhow!("update module unloading failed: {e:#}"))?;

    self.module = Some(Self::load_()?);

    Ok(())
  }

  pub unsafe fn update(&self, state: *mut ()) {
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
