use libloading::Library;

pub trait ModuleExportsForHost {
  fn new(library: &Library) -> Self;
}

/// For cases when module doesn't export anything
impl ModuleExportsForHost for () {
  fn new(_library: &Library) {}
}

pub trait InitImports {
  fn init(self, library: &Library);
}

impl<F> InitImports for F
where
  F: FnOnce(&Library),
{
  fn init(self, library: &Library) {
    self(library)
  }
}

/// For cases when module doesn't import anything
impl InitImports for () {
  fn init(self, _library: &Library) {}
}
