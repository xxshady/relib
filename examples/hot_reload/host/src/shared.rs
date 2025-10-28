use {
  libloading::library_filename,
  relib_host::{InitImports, Module, ModuleExportsForHost},
  std::{fs, path::Path},
};

pub type AnyErrorResult<T = ()> = anyhow::Result<T>;

pub fn load_module<E: ModuleExportsForHost>(
  name: &str,
  init_imports: impl InitImports,
  enable_alloc_tracker: bool,
) -> AnyErrorResult<Module<E>> {
  let dylib_filename = library_filename(name);
  let dylib_copy_filename = library_filename(format!("copy_{name}"));
  let dylib_path = Path::new("target/debug").join(dylib_filename);
  let dylib_copy_path = Path::new("target/debug").join(dylib_copy_filename);

  fs::copy(&dylib_path, &dylib_copy_path)?;

  let module = unsafe {
    relib_host::load_module_with_options::<E>(dylib_copy_path, init_imports, enable_alloc_tracker)?
  };

  Ok(module)
}
