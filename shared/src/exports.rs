use crate::{ModuleId, SliceAllocation};

#[expect(non_camel_case_types)]
pub trait ___Internal___Exports___ {
  fn init(host_owner_thread: usize, module: ModuleId);
  fn exit(allocs: SliceAllocation);
  fn take_cached_allocs_before_exit();
  fn lock_module_allocator();

  // currently linux-only
  fn spawned_threads_count() -> u64;
  fn run_thread_local_dtors();
}
