use std::env::consts::{DLL_PREFIX, DLL_SUFFIX};

macro_rules! cmd_impl {
  ( $program:expr $(, $arg:expr )* $(; current_dir: $current_dir:expr )? ) => {
    (
      || {
        $crate::helpers::cmd!(
          @impl $program $(, $arg )* $(; current_dir: $current_dir )?
        );
      },
      || {
        $crate::helpers::cmd!(
          @impl $program $(, $arg )*, "--release" $(; current_dir: $current_dir )?
        );
      }
    )
  };

  ( @impl $program:expr $(, $arg:expr )* $(; current_dir: $current_dir:expr )? ) => {
    let args: &[String] = &[ $( $arg.clone().into(), )* ];

    let full_command = {
      let args = args.join(" ");
      let args = if args.is_empty() {
        "".to_owned()
      } else {
        format!(" {args}")
      };

      format!("`{}{args}`", $program)
    };
    println!("running {full_command}");

    let status = std::process::Command::new($program)
      .args(args)
      $( .current_dir($current_dir) )?
      .status()
      .unwrap_or_else(|e| {
        panic!("failed to execute: {full_command}, reason: {e:#?}");
      });

    if !status.success() {
      panic!("{full_command} did not exit successfully");
    }
  };
}

pub(crate) use cmd_impl as cmd;

pub fn host_bin_by_directory(directory: &str) -> String {
  if cfg!(target_os = "linux") {
    format!("target/{directory}/test_host")
  } else {
    format!("target/{directory}/test_host.exe")
  }
}

pub fn call_host_by_directory(directory: &str) {
  let (host_bin, _) = cmd!(&host_bin_by_directory(directory));
  host_bin();
}

// TODO: use libloading::library_filename?
pub fn dylib_filename(name: &str) -> String {
  format!("{DLL_PREFIX}{name}{DLL_SUFFIX}")
}
