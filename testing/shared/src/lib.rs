pub mod exports;
pub mod imports;

pub const EXPORTS: &str = include_str!("exports.rs");
pub const IMPORTS: &str = include_str!("imports.rs");

pub const SIZE_200_MB: usize = 1024 * 1024 * 200;

pub fn print_memory_use() {
  let (_, megabytes) = memory_use();
  println!("[host] memory in use: {megabytes:.2}mb");
}

pub fn memory_use() -> (usize, f64) {
  let stats = memory_stats::memory_stats().unwrap();
  let bytes = stats.virtual_mem;
  let megabytes = (bytes as f64) / 1024. / 1024.;

  (bytes, megabytes)
}

pub fn assert_mem_dealloc<R>(f: impl FnOnce() -> R) -> R {
  let (before_mem, _) = memory_use();
  let returned = f();
  let (after_mem, _) = memory_use();
  assert_eq!(after_mem, before_mem, "memory must be deallocated");

  returned
}
