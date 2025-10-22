mod allocator_proxy;

use relib_module as _;

use state::State;
use update_contract::exports::Exports;

relib_interface::include_exports!(gen_exports, "update");
use gen_exports::ModuleExportsImpl;
relib_interface::include_imports!(gen_imports, "update");

impl Exports for ModuleExportsImpl {
  fn main_contract_build_id() -> u128 {
    main_contract::build_id()
  }

  fn update(state: *mut ()) {
    let state = unsafe {
      // SAFETY: we logically receive state with exclusive access,
      // can't actually pass it as &mut because relib requires parameters
      // to be Copy to prevent module allocator touching foreign memory.
      // here it's safe because this module shares global allocator with
      // main module (see allocator_proxy.rs) and this state is only
      // mutated in this module.
      &mut *(state as *mut State)
    };

    println!("update {}", state.foo);

    let foo = unsafe { gen_imports::foo() };
    println!("foo call: {foo:?}");

    state.foo += 1;
    for _ in 1..=(1024 * 1024 * 1) {
      state.bar.push(1);
    }
    // state.bar = vec![];
  }
}

#[relib_module::export]
fn before_unload() {
  println!("[update] before unload");
}
