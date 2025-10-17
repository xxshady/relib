use std::alloc::{Layout, System, alloc, dealloc};
use main_contract::{StableLayout, exports::Exports};
use relib_module::AllocTracker;
use state::State;

relib_interface::include_exports!();
use gen_exports::ModuleExportsImpl;
relib_interface::include_imports!();

impl Exports for ModuleExportsImpl {
  fn main_contract_build_id() -> u128 {
    main_contract::build_id()
  }

  fn call_alloc(layout: StableLayout) -> *mut u8 {
    unsafe { alloc(Layout::from_size_align(layout.size, layout.align).unwrap()) }
  }

  fn call_dealloc(ptr: *mut u8, layout: StableLayout) {
    unsafe {
      dealloc(
        ptr,
        Layout::from_size_align(layout.size, layout.align).unwrap(),
      )
    }
  }
}

#[expect(unreachable_code)]
pub fn _suppress_unused_warnings() {
  let _ = unsafe { gen_imports::alloc(unreachable!()) };
  let _ = unsafe { gen_imports::dealloc(unreachable!(), unreachable!()) };
}

// workaround for cargo feature unification
#[global_allocator]
static ALLOC: AllocTracker<System> = AllocTracker::new(System);

#[relib_module::export]
fn main() -> *mut () {
  let foo_ = unsafe { gen_imports::foo() };
  println!("change me! {foo_} but don't touch main_contract crate");

  let state = State {
    foo: 1,
    bar: vec![1, 2, 3],
    baz: 2,
  };
  Box::into_raw(Box::new(state)) as *mut ()
}
