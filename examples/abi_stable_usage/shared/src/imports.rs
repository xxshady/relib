use abi_stable::std_types::RVec;

pub trait Imports {
  fn foo() -> RVec<u8>;
}
