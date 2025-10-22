#![allow(unused_imports)] // TODO: fix this

mod shared;
mod update_instance;

use std::{
  cell::{Cell, RefCell},
  env,
  error::Error,
  path::Path,
  process::Command,
  rc::Rc,
  thread,
  time::{Duration, Instant},
};
use anyhow::{anyhow, bail};
use shared::AnyErrorResult;
use relib_host::{Module};
use main_contract::{imports::Imports, StableLayout};

relib_interface::include_exports!();
use gen_exports::ModuleExports;

relib_interface::include_imports!();
use gen_imports::{init_imports, ModuleImportsImpl as MainModuleImportsImpl};

use crate::{shared::load_module, update_instance::UpdateModule};

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
  let (main_module, mut state) = run_main_module()?;
  let mut main_module = Rc::new(RefCell::new(Some(main_module)));
  set_alloc_and_dealloc(main_module.clone());

  let mut update_module = UpdateModule::load()?;

  let mut build_failed_in_prev_iteration = false;
  loop {
    let build_res = cargo_build(&["module", "update"])?;
    match build_res {
      BuildResult::Success(modules) => {
        build_failed_in_prev_iteration = false;

        let main_module_reload = modules.contains(&"module");

        if main_module_reload {
          println!("main module has been rebuilt");

          let main_module_ = main_module.borrow_mut().take().unwrap();

          // when unloading fails it is not safe to load it again
          main_module_
            .unload()
            .map_err(|e| anyhow!("module unloading failed: {e:#}"))?;

          let (main_module_, state_) = run_main_module()?;
          main_module = Rc::new(RefCell::new(Some(main_module_)));
          state = state_;
          set_alloc_and_dealloc(main_module.clone());
        }

        match (modules.contains(&"update"), main_module_reload) {
          (true, _) => {
            println!("update module has been rebuilt");
            update_module.reload()?;
          }
          (_, true) => {
            // since main module shares global allocator with update module,
            // we need to reload it too since main module deallocated everything
            // and update module now may have dangling pointers (it was really fun to debug)
            println!("reloading update module due to main module reload");
            update_module.reload()?;
          }
          _ => {}
        }
      }
      BuildResult::Failure(modules) => {
        if build_failed_in_prev_iteration {
          continue;
        }
        build_failed_in_prev_iteration = true;

        println!("failed to build modules:\n{modules:?}");
        if modules.is_empty() {
          println!("note: failed to compile dependency of host/module/update crate")
        }
      }
      BuildResult::NoChange => {}
    }

    if !build_failed_in_prev_iteration {
      unsafe {
        update_module.update(state);
      }
    }

    thread::sleep(Duration::from_millis(50));
  }
}

pub fn run_main_module() -> AnyErrorResult<(Module<ModuleExports>, *mut ())> {
  let module: Module<ModuleExports> = load_module("module", init_imports, true)?;

  let module_main_contract_build_id = unsafe { module.exports().main_contract_build_id() }.unwrap();
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

// TODO: use json format of cargo build?
fn cargo_build<'a>(modules: &'a [&'a str]) -> AnyErrorResult<BuildResult<'a>> {
  let mut command = Command::new("cargo");

  command
    .arg("build")
    .env("CARGO_LOG", "cargo::core::compiler::fingerprint=info");

  let output = command.output()?;

  let stderr = String::from_utf8(output.stderr)?;

  if stderr.contains("Compiling host") {
    bail!(
      "host was recompiled, if contracts were modified it potentially contains old incompatible code"
    );
  }

  if !output.status.success() {
    let mut failed_modules = Vec::new();
    for name in modules {
      if stderr.contains(&format!("error: could not compile `{name}`")) {
        failed_modules.push(*name);
      }
    }

    return Ok(BuildResult::Failure(failed_modules));
  }

  if !stderr.contains("Compiling") {
    return Ok(BuildResult::NoChange);
  }

  let mut success_modules = Vec::new();
  for name in modules {
    if stderr.contains(&format!("Compiling {name}")) {
      success_modules.push(*name);
    }
  }

  Ok(BuildResult::Success(success_modules))
}

enum BuildResult<'a> {
  Success(Vec<&'a str>),
  Failure(Vec<&'a str>),
  NoChange,
}
