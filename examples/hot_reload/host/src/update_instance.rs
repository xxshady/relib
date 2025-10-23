relib_interface::include_exports!(gen_exports, "update");
use anyhow::anyhow;
use gen_exports::ModuleExports;

relib_interface::include_imports!(gen_imports, "update");
use gen_imports::{init_imports, ModuleImportsImpl};
use main_contract::{Alloc, Dealloc, StableLayout, shared_imports::SharedImports};
use relib_host::Module;

use crate::{
  MainModuleImportsImpl,
  shared::{AnyErrorResult, load_module},
};

impl SharedImports for ModuleImportsImpl {
  fn spawn_entity_from_not_perfect_parallel_universe() -> u64 {
    MainModuleImportsImpl::spawn_entity_from_not_perfect_parallel_universe()
  }

  fn despawn_entity_from_not_perfect_parallel_universe(entity: u64) {
    MainModuleImportsImpl::despawn_entity_from_not_perfect_parallel_universe(entity)
  }
}

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
    let module: Module<ModuleExports> = load_module("update", init_imports, false)?;

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
