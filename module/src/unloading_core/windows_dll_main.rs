use std::ffi::c_void;

use super::windows_dealloc;

#[expect(clippy::upper_case_acronyms)]
type BOOL = i32;
pub const DLL_PROCESS_DETACH: u32 = 0;
const TRUE: i32 = 1;

#[unsafe(no_mangle)]
unsafe extern "system" fn DllMain(
  _module_handle: *mut c_void,
  reason: u32,
  lpv_reserved: *mut c_void,
) -> BOOL {
  windows_dealloc::on_dll_main_call(reason, lpv_reserved);

  TRUE
}
