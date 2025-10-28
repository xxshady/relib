use {
  crate::shared::current_target_dir,
  libloading::{Library, library_filename},
};

pub fn main() {
  let path = current_target_dir()
    .join("backtrace_unloading_host_as_dylib__host")
    .join(library_filename("test_host_as_dylib"));

  unsafe {
    let host = Library::new(path).unwrap();
    let symbol = host.get(b"main\0").unwrap();
    let main: extern "C" fn() = *symbol;
    main();
  }
}
