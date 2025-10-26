mod shared;
mod main_instance;
mod update_instance;
mod imperfect_api_impl;

use {
  crate::{main_instance::MainModule, update_instance::UpdateModule},
  anyhow::bail,
  shared::AnyErrorResult,
  std::{
    process::Command,
    thread,
    time::Duration,
  },
};

fn main() {
  if let Err(e) = main_fallible() {
    eprintln!(
      "main exited with error:\n\
      {e:?}"
    );
  }
}

fn main_fallible() -> AnyErrorResult {
  let mut main_module = MainModule::load()?;
  let mut update_module = UpdateModule::load(main_module.ret.alloc, main_module.ret.dealloc)?;

  let mut build_failed_in_prev_iteration = false;
  loop {
    let build_res = cargo_build(&["main_module", "update_module"])?;
    match build_res {
      BuildResult::Success(modules) => {
        build_failed_in_prev_iteration = false;

        let main_module_reload = modules.contains(&"main_module");

        if main_module_reload {
          println!("main module has been rebuilt");

          // leaks are safe in rust
          imperfect_api_impl::despawn_leaked_entities();

          main_module.reload()?;
        }

        let update_module_reload = modules.contains(&"update_module");

        match (update_module_reload, main_module_reload) {
          (true, _) => {
            println!("update module has been rebuilt");
          }
          (_, true) => {
            // since main module shares global allocator with update module,
            // we need to reload it and call startup since main module deallocated everything
            // and update module now may have dangling pointers (it was really fun to debug)
            println!("reloading update module due to main module reload");
          }
          _ => {}
        }

        if let (true, _) | (_, true) = (update_module_reload, main_module_reload) {
          update_module.reload(main_module.ret.alloc, main_module.ret.dealloc)?;
        }
      }
      BuildResult::Failure(modules) => {
        if build_failed_in_prev_iteration {
          continue;
        }
        build_failed_in_prev_iteration = true;

        println!("failed to build modules:\n{modules:?}");
        if modules.is_empty() {
          println!("note: failed to compile dependency of host/main_module/update_module crate")
        }
      }
      BuildResult::NoChange => {}
    }

    if !build_failed_in_prev_iteration {
      unsafe {
        update_module.update(main_module.ret.state);
      }
    }

    thread::sleep(Duration::from_millis(350));
  }
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
