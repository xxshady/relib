mod allocator_proxy;

use relib_module as _;

use state::State;
use update_contract::exports::Exports;

relib_interface::include_exports!(gen_exports, "update");
use gen_exports::ModuleExportsImpl;
relib_interface::include_imports!(gen_imports, "update");

impl Exports for ModuleExportsImpl {
  fn update(state: *mut State) {
    // TODO: SAFETY
    let state = unsafe { &mut *state };
    println!("update {}", state.foo);

    let foo = unsafe { gen_imports::foo() };
    println!("foo call: {foo:?}");

    state.foo += 1;
    for _ in 1..=(1024 * 1024) {
      state.bar.push(1);
    }
    // state.bar = vec![];
  }
}
