#![allow(unused_imports)] // TODO: fix this

mod shared;
mod update_instance;

use std::{
  cell::{Cell, RefCell},
  error::Error,
  process::Command,
  rc::Rc,
  thread,
  time::{Duration, Instant},
};
use anyhow::anyhow;
use shared::{build_module, measure_time, AnyErrorResult};
use relib_host::{Module};
use main_contract::{imports::Imports, StableLayout};

relib_interface::include_exports!();
use gen_exports::ModuleExports;

relib_interface::include_imports!();
use gen_imports::{init_imports, ModuleImportsImpl as MainModuleImportsImpl};
use state::State;

use crate::{
  shared::{BuildResult, load_module},
  update_instance::UpdateModule,
};

impl Imports for MainModuleImportsImpl {
  fn foo() -> i32 {
    123
  }

  fn alloc(_layout: StableLayout) -> *mut u8 {
    unreachable!()
  }

  fn dealloc(_ptr: *mut u8, _layout: StableLayout) {
    unreachable!()
  }
}

// TODO: this shit is ugly as hell
thread_local! {
  static CALL_MAIN_MODULE_ALLOC: RefCell<Box<dyn Fn(StableLayout) -> *mut u8>> = {
    let f = |_| { panic!("call_main_module_alloc not initialized") };

    RefCell::new(Box::new(f))
  };
  static CALL_MAIN_MODULE_DEALLOC: RefCell<Box<dyn Fn(*mut u8, StableLayout)>> = {
    let f = |_, _| { panic!("call_main_module_dealloc not initialized") };

    RefCell::new(Box::new(f))
  };
}

fn set_alloc_and_dealloc(module: Rc<RefCell<Option<Module<ModuleExports>>>>) {
  CALL_MAIN_MODULE_ALLOC.set({
    let module = module.clone();
    let f = move |layout| {
      let module = module.borrow();
      let module = module.as_ref().unwrap();
      unsafe { module.exports().call_alloc(layout) }.unwrap()
    };

    Box::new(f)
  });

  CALL_MAIN_MODULE_DEALLOC.set({
    let module = module.clone();
    let f = move |ptr, layout| {
      let module = module.borrow();
      let module = module.as_ref().unwrap();
      unsafe { module.exports().call_dealloc(ptr, layout) }.unwrap()
    };

    Box::new(f)
  });
}

fn main() {
  if let Err(e) = main_fallible() {
    eprintln!(
      "main exited with error:\n\
      {e:?}"
    );
  }
}

fn main_fallible() -> AnyErrorResult {
  let (module, mut state) = run_main_module()?;
  let mut module = Rc::new(RefCell::new(Some(module)));
  set_alloc_and_dealloc(module.clone());

  let mut update_module = UpdateModule::load()?;

  let mut build_failed_in_prev_iteration = false;
  loop {
    let build_res = measure_time("building", || build_module("module"))?;
    match build_res {
      BuildResult::Success => {
        let module_ = module.borrow_mut().take().unwrap();

        // when unloading fails it is not safe to load it again
        measure_time("unloading", || {
          module_
            .unload()
            .map_err(|e| anyhow!("module unloading failed: {e:#}"))
        })?;

        println!("main module has been rebuilt");

        // inserting new line for more clear output of module after compilation failures or previous runs of the module
        println!();

        let (module_, state_) = run_main_module()?;
        module = Rc::new(RefCell::new(Some(module_)));
        state = state_;
        set_alloc_and_dealloc(module.clone());

        build_failed_in_prev_iteration = false;
      }
      BuildResult::Failure(message) => {
        if build_failed_in_prev_iteration {
          continue;
        }
        build_failed_in_prev_iteration = true;

        println!("failed to build the module:\n{message}");
      }
      BuildResult::NoChange => {}
    }

    if !build_failed_in_prev_iteration {
      update_module.rebuild()?;
      unsafe {
        let state = std::mem::transmute::<*mut (), *mut State>(state);
        update_module.update(state);
      }
    }

    thread::sleep(Duration::from_millis(50));
  }
}

pub fn run_main_module() -> AnyErrorResult<(Module<ModuleExports>, *mut ())> {
  let module: Module<ModuleExports> = load_module("module", init_imports)?;

  let module_main_contract_build_id = measure_time("getting main_contract build id", || {
    unsafe { module.exports().main_contract_build_id() }.unwrap()
  });
  let host_main_contract_build_id = main_contract::build_id();

  // when main_contract crate is modified it's no longer safe to load the module,
  // so we need to stop here
  if module_main_contract_build_id != host_main_contract_build_id {
    return Err(anyhow!(
      "main_contract crate was modified, module potentially contains incompatible code\n\
        main_contract build id of:\n\
        host:   {}\n\
        module: {}",
      host_main_contract_build_id,
      module_main_contract_build_id
    ));
  }

  // state is opaque pointer here because it's owned by main module allocator
  // (it will deallocate it at unloading) and host should not mutate it
  let state: *mut () = unsafe { module.call_main().unwrap() };
  Ok((module, state))
}
