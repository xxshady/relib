use std::{ffi::OsString, os::windows::ffi::OsStringExt, path::PathBuf};

use minhook::MinHook;

/// What does it solve:
/// 1. Synchronizes dbghelp.dll between all modules: each module has it's own standard library and because of that sync
///    of dbghelp.dll in backtrace crate (which is used by std) doesn't work (dbghelp.dll is single-threaded)
/// 2. Fixes unloading of dynamic libraries from dbghelp.dll
// TODO: load dbghelp.dll lazily when std or backtrace crate calls LoadLibrary?
// TODO: crate feature to disable dbghelp stuff if backtraces are not needed
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

  windows_targets::link!("kernel32.dll" "system" fn GetModuleHandleExW(flags: u32, module_name: *const u16, module: *mut *mut isize) -> BOOL);
  windows_targets::link!("kernel32.dll" "system" fn GetModuleFileNameW(module: *const isize, file_name: PWSTR, size: DWORD) -> DWORD);

  windows_targets::link!("kernel32.dll" "system" fn GetCurrentProcess() -> HANDLE);
  windows_targets::link!("kernel32.dll" "system" fn GetLastError() -> DWORD);

  #[cfg(feature = "unloading")]
  windows_targets::link!("kernel32.dll" "system" fn CreateThread(lpthreadattributes : *const c_void, dwstacksize : usize, lpstartaddress : unsafe extern "system" fn(main: *mut c_void) -> u32, lpparameter : *mut c_void, dwcreationflags : u32, lpthreadid : *mut u32) -> HANDLE);
  #[cfg(feature = "unloading")]
  pub type CreateThread = unsafe extern "system" fn(
    lpthreadattributes: *const c_void,
    dwstacksize: usize,
    lpstartaddress: unsafe extern "system" fn(main: *mut c_void) -> u32,
    lpparameter: *mut c_void,
    dwcreationflags: u32,
    lpthreadid: *mut u32,
  ) -> HANDLE;
}
use imports::{
  GetLastError, GetModuleFileNameW, GetModuleHandleExW, DWORD, ERROR_INSUFFICIENT_BUFFER,
  GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS, GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
};

pub fn str_to_wide_cstring(str: &str) -> Vec<u16> {
  str.bytes().map(|b| b as u16).chain(Some(0)).collect()
}

pub fn get_current_dylib() -> Option<PathBuf> {
  fn get_dylib_path(len: usize) -> Option<PathBuf> {
    let mut buf = Vec::with_capacity(len);
    unsafe {
      let module_handle = get_dylib_handle_from_addr(get_dylib_path as *const _)?;
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

  get_dylib_path(100)
}

pub unsafe fn get_dylib_handle_from_addr(addr: *const u16) -> Option<*mut isize> {
  let mut module_handle = std::ptr::null_mut();

  let flags = GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS | GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT;
  let ret = unsafe { GetModuleHandleExW(flags, addr, &mut module_handle) };

  let failed = ret == 0;
  if failed { None } else { Some(module_handle) }
}

pub unsafe fn enable_hooks() {
  let res = unsafe { MinHook::enable_all_hooks() };
  res.unwrap_or_else(|e| {
    panic!("Failed to enable Windows related hooks: {e:?}");
  });
}
