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
use main_contract::{shared_imports::SharedImports, MainModuleRet, StableLayout};

relib_interface::include_imports!();
use gen_imports::{init_imports, ModuleImportsImpl as MainModuleImportsImpl};

use crate::{shared::load_module, update_instance::UpdateModule};

impl SharedImports for MainModuleImportsImpl {
  fn spawn_entity_from_not_perfect_parallel_universe() -> u64 {
    let entity = 123;
    println!("spawning entity {entity} from unperfect universe");
    entity
  }

  fn despawn_entity_from_not_perfect_parallel_universe(entity: u64) {
    println!("despawning entity {entity} from unperfect universe");
  }
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
  let (main_module, mut ret) = run_main_module()?;
  let mut main_module = Some(main_module);

  let mut update_module = UpdateModule::load(ret.alloc, ret.dealloc)?;

  let mut build_failed_in_prev_iteration = false;
  loop {
    let build_res = cargo_build(&["module", "update"])?;
    match build_res {
      BuildResult::Success(modules) => {
        build_failed_in_prev_iteration = false;

        let main_module_reload = modules.contains(&"module");

        if main_module_reload {
          println!("main module has been rebuilt");

          let main_module_ = main_module.take().unwrap();
          // when unloading fails it is not safe to load it again
          main_module_
            .unload()
            .map_err(|e| anyhow!("module unloading failed: {e:#}"))?;

          let (main_module_, ret_) = run_main_module()?;
          ret = ret_;
          main_module = Some(main_module_);
        }

        match (modules.contains(&"update"), main_module_reload) {
          (true, _) => {
            println!("update module has been rebuilt");
            update_module.reload(ret.alloc, ret.dealloc)?;
          }
          (_, true) => {
            // since main module shares global allocator with update module,
            // we need to reload it and call startup since main module deallocated everything
            // and update module now may have dangling pointers (it was really fun to debug)
            println!("reloading update module due to main module reload");
            update_module.reload(ret.alloc, ret.dealloc)?;
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
        update_module.update(ret.state);
      }
    }

    thread::sleep(Duration::from_millis(350));
  }
}

pub fn run_main_module() -> AnyErrorResult<(Module<()>, MainModuleRet)> {
  let module: Module<()> = load_module("module", init_imports, true)?;
  let ret: MainModuleRet = unsafe { module.call_main().unwrap() };
  Ok((module, ret))
}

// TODO: use json format of cargo build?
fn cargo_build<'a>(modules: &'a [&'a str]) -> AnyErrorResult<BuildResult<'a>> {
  let output = Command::new("cargo").arg("build").output()?;
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
