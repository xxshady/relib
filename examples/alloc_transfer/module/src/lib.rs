// TODO: remove it?
mod transfer;

use shared::Exports;

relib_interface::include_all!();

impl Exports for gen_exports::ModuleExportsImpl {
  fn take_forget(vec: Vec<u8>) {
    dbg!(&vec[0..10], vec.as_ptr());
    std::mem::forget(vec);
  }

  fn take_drop(vec: Vec<u8>) {
    dbg!(&vec[0..10], vec.as_ptr());
    drop(vec);
  }

  fn ret() -> Vec<u8> {
    let mut vec = Vec::<u8>::with_capacity(1024 * 1024 * 100);
    for _ in 1..=(1024 * 1024 * 100) {
      vec.push(2);
    }
    vec
  }
}

#[relib_module::export]
fn main() -> Vec<u8> {
  let mut vec = Vec::<u8>::with_capacity(1024 * 1024 * 100);
  for _ in 1..=(1024 * 1024 * 100) {
    vec.push(2);
  }

  dbg!(vec.len());

  unsafe {
    gen_imports::take_drop(vec.clone());
    let vec = gen_imports::ret();
    std::mem::forget(vec);
  }

  vec
}
