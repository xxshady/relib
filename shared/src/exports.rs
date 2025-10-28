use {
  crate::{ModuleId, SliceAllocation},
  std::ffi::c_void,
};

#[expect(non_camel_case_types)]
pub trait ___Internal___Exports___ {
  fn init(host_owner_thread: usize, module: ModuleId, enable_alloc_tracker: bool);
  fn exit(allocs: SliceAllocation);
  fn take_cached_allocs_before_exit();
  fn lock_module_allocator();

  // linux-only
  fn spawned_threads_count() -> u64;
  fn run_thread_local_dtors();
  fn misc_cleanup();

  // windows-only
  fn set_dealloc_callback(callback: *const c_void);
}
