use std::{fmt::Debug, marker::PhantomData};

#[cfg(feature = "unloading")]
use std::path::PathBuf;

use relib_internal_shared::ModuleId;
use libloading::Library;

use crate::{
  exports_types::ModuleExportsForHost, helpers::call_module_pub_export, leak_library::LeakLibrary,
};

#[cfg(feature = "unloading")]
use crate::unloading::InternalModuleExports;

#[must_use = "module will be leaked if dropped, \
  if you don't want that consider using `unload` method (see \"unloading\" feature)"]
pub struct Module<E: ModuleExportsForHost> {
  pub id: ModuleId,
  pub(crate) library: LeakLibrary,

  #[cfg(feature = "unloading")]
  pub(crate) library_path: PathBuf,

  #[cfg(feature = "unloading")]
  pub(crate) internal_exports: InternalModuleExports,

  pub_exports: E,

  /// Module must be loaded and unloaded from the same thread
  _not_thread_safe: PhantomData<*const ()>,
}

impl<E: ModuleExportsForHost> Module<E> {
  pub(crate) fn new(
    id: ModuleId,
    library: Library,
    pub_exports: E,

    #[cfg(feature = "unloading")] (internal_exports, library_path): (
      InternalModuleExports,
      PathBuf,
    ),
  ) -> Self {
    Self {
      id,
      library: LeakLibrary::new(library),
      pub_exports,
      _not_thread_safe: PhantomData,

      #[cfg(feature = "unloading")]
      library_path,
      #[cfg(feature = "unloading")]
      internal_exports,
    }
  }

  pub fn library(&self) -> &Library {
    self.library.get_ref()
  }

  pub fn exports(&self) -> &E {
    &self.pub_exports
  }

  /// Returns `None` if module panics.
  /// Consider unloading module if it panicked, as it is unsafe to call it again.
  /// Note: not all panics are handled, see a ["double panic"](https://doc.rust-lang.org/std/ops/trait.Drop.html#panics)
  /// ```
  /// struct Bomb;
  /// impl Drop for Bomb {
  ///   fn drop(&mut self) {
  ///     panic!("boom"); // will abort the program
  ///   }
  /// }
  /// let _bomb = Bomb;
  /// panic!();
  /// ```
  ///
  /// # Safety
  /// Behavior is undefined if any of the following conditions are violated:
  /// 1. Returned value must be actually `R` at runtime. For example if you called this function with type `bool` but module returns `i32`, UB will occur.
  /// 2. Type of return value must be FFI-safe.
  /// 3. Returned value must not be a reference-counting pointer (see [caveats](https://docs.rs/relib/latest/relib/#moving-non-copy-types-between-host-and-module)).
  ///
  /// # Panics
  /// If main function is not exported from the module.
  #[must_use = "returns `None` if module panics"]
  pub unsafe fn call_main<R>(&self) -> Option<R>
  where
    R: Clone,
  {
    let res = unsafe { call_module_pub_export(self.library(), "main") };
    res.unwrap_or_else(|e| {
      panic!("Failed to get main fn from module, reason: {e:#}");
    })
  }
}

impl<E: ModuleExportsForHost> Debug for Module<E> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let id = self.id;
    write!(f, "Module {{ id: {id} }}")
  }
}
