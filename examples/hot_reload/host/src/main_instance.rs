use {
  crate::{
    imperfect_api_impl::init_shared_imports,
    shared::{AnyErrorResult, load_module},
    update_instance::UpdateModule,
  },
  anyhow::{anyhow, bail},
  main_contract::{MainModuleRet, SharedImports, StableLayout},
  relib_host::Module,
  std::{
    cell::{Cell, RefCell},
    env,
    error::Error,
    path::Path,
    process::Command,
    rc::Rc,
    thread,
    time::{Duration, Instant},
  },
};

relib_interface::include_exports!(gen_exports, "main_module");
use gen_exports::ModuleExports;

pub struct MainModule {
  module: Option<Module<ModuleExports>>,
  pub ret: MainModuleRet,
}

impl MainModule {
  pub fn load() -> AnyErrorResult<Self> {
    println!("loading main module");

    let module = load_module("main_module", init_shared_imports, true)?;
    let ret = unsafe { module.call_main().unwrap() };

    Ok(Self {
      module: Some(module),
      ret,
    })
  }

  pub fn reload(&mut self) -> AnyErrorResult<()> {
    let module = self.module.take().unwrap();

    unsafe {
      module.exports().drop_state(self.ret.state).unwrap();
    }

    // when unloading fails it is not safe to load it again
    module
      .unload()
      .map_err(|e| anyhow!("main module unloading failed: {e:#}"))?;

    let new_module = Self::load()?;
    self.module = new_module.module;
    self.ret = new_module.ret;

    Ok(())
  }
}
