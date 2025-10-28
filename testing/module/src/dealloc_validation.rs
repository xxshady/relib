use std::alloc::{Layout, dealloc};

#[relib_module::export]
pub fn main() {
  unsafe {
    dealloc(0x1 as *mut u8, Layout::new::<u8>());
  }
}
