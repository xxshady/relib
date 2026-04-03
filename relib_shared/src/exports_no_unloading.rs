use crate::{Alloc, Dealloc};

#[expect(non_camel_case_types)]
pub trait ___Exports___NoUnloading___ {
  fn init(alloc: Alloc, dealloc: Dealloc);
}
