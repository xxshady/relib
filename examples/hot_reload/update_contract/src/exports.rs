use main_contract::{Alloc, Dealloc};

pub trait Exports {
  fn init_allocator_proxy(alloc: Alloc, dealloc: Dealloc);
  fn update(state: *mut ());
}
