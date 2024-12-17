use libloading::Library;

/// Leak library if it was implicitly dropped without calling `take`
pub struct LeakLibrary(Option<Library>);

impl LeakLibrary {
  pub fn new(library: Library) -> Self {
    Self(Some(library))
  }

  pub fn get_ref(&self) -> &Library {
    self.0.as_ref().unwrap_or_else(|| unreachable!())
  }

  pub fn take(mut self) -> Library {
    self.0.take().unwrap_or_else(|| unreachable!())
  }
}

impl Drop for LeakLibrary {
  fn drop(&mut self) {
    if let Some(library) = self.0.take() {
      std::mem::forget(library);
    }
  }
}
