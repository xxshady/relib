relib_interface::include_exports!(gen_exports, "update");
use anyhow::anyhow;
use gen_exports::ModuleExports;

use main_contract::{Alloc, Dealloc, StableLayout};
use relib_host::Module;

use crate::{
  shared::{AnyErrorResult, load_module},
  unperfect_api_bindings::init_shared_imports,
};

pub struct UpdateModule {
  module: Option<Module<ModuleExports>>,
}

impl UpdateModule {
  pub fn load(alloc: Alloc, dealloc: Dealloc) -> AnyErrorResult<UpdateModule> {
    let module = Self::load_(alloc, dealloc)?;
    Ok(Self {
      module: Some(module),
    })
  }

  fn load_(alloc: Alloc, dealloc: Dealloc) -> AnyErrorResult<Module<ModuleExports>> {
    let module: Module<ModuleExports> = load_module("update", init_shared_imports, false)?;

    unsafe {
      module
        .exports()
        .init_allocator_proxy(alloc, dealloc)
        .unwrap()
    }

    Ok(module)
  }

  pub fn reload(&mut self, alloc: Alloc, dealloc: Dealloc) -> AnyErrorResult<()> {
    // when unloading fails it is not safe to load it again
    self
      .module
      .take()
      .unwrap()
      .unload()
      .map_err(|e| anyhow!("update module unloading failed: {e:#}"))?;

    self.module = Some(Self::load_(alloc, dealloc)?);

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
