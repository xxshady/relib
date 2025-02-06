use std::{ffi::OsString, os::windows::ffi::OsStringExt, path::PathBuf};

/// What does it solve:
/// 1. Synchronizes dbghelp.dll between all modules: each module has it's own standard library and because of that sync
///    of dbghelp.dll in backtrace crate (which is used by std) doesn't work (dbghelp.dll is single-threaded)
/// 2. Fixes unloading of dynamic libraries from dbghelp.dll
// TODO: load dbghelp.dll lazily when std or backtrace crate calls LoadLibrary?
// TODO: crate feature to disable dbghelp stuff if backtraces are not needed
#[cfg(target_os = "windows")]
pub mod dbghelp;

#[allow(clippy::upper_case_acronyms)]
pub mod imports {
  use std::ffi::c_void;

  pub type HANDLE = *mut c_void;
  pub type BOOL = i32;
  pub const TRUE: BOOL = 1;
  pub const FALSE: BOOL = 0;
  pub type PCWSTR = *const u16;
  pub type PWSTR = *mut u16;
  pub type DWORD = u32;

  pub const GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT: u32 = 0x00000002;
  pub const GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS: u32 = 0x00000004;
  pub const ERROR_INSUFFICIENT_BUFFER: u32 = 122;

  windows_targets::link!("kernel32.dll" "system" fn GetModuleHandleExW(flags: u32, module_name: *const u16, module: *mut isize) -> BOOL);
  windows_targets::link!("kernel32.dll" "system" fn GetModuleFileNameW(module: *const isize, file_name: PWSTR, size: DWORD) -> DWORD);

  windows_targets::link!("kernel32.dll" "system" fn GetCurrentProcess() -> HANDLE);
  windows_targets::link!("kernel32.dll" "system" fn GetLastError() -> DWORD);
}
use imports::{
  GetLastError, GetModuleFileNameW, GetModuleHandleExW, DWORD, ERROR_INSUFFICIENT_BUFFER,
  GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS, GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
};

pub fn str_to_wide_cstring(str: &str) -> Vec<u16> {
  str.bytes().map(|b| b as u16).chain(Some(0)).collect()
}

// copy-pasted from process_path crate
pub fn get_current_dylib() -> Option<PathBuf> {
  fn get_dylib_path(len: usize) -> Option<PathBuf> {
    let mut buf = Vec::with_capacity(len);
    unsafe {
      // TODO: test it
      let module_handle = std::ptr::null_mut();

      let flags =
        GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS | GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT;
      let failed = GetModuleHandleExW(flags, get_dylib_path as *const _, module_handle) == 0;
      if failed {
        None
      } else {
        let ret = GetModuleFileNameW(module_handle, buf.as_mut_ptr(), len as DWORD) as usize;
        if ret == 0 {
          None
        } else if ret < len {
          // Success, we need to trim trailing null bytes from the vec.
          buf.set_len(ret);
          let s = OsString::from_wide(&buf);
          Some(s.into())
        } else {
          // The buffer might not be big enough so we need to check errno.
          let errno = GetLastError();
          if errno == ERROR_INSUFFICIENT_BUFFER {
            get_dylib_path(len * 2)
          } else {
            None
          }
        }
      }
    }
  }

  get_dylib_path(100)
}
