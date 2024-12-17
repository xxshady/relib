use std::{fmt::Debug, marker::PhantomData, path::PathBuf};

use relib_internal_shared::ModuleId;
use libloading::Library;

use crate::{
  errors::UnloadError,
  exports_types::{ModuleExportsForHost, ModuleValue},
  gen_exports::ModuleExports as InternalModuleExports,
  helpers::call_module_pub_export,
  leak_library::LeakLibrary,
  module_allocs,
};

#[must_use = "module will be leaked if dropped, if you don't want that consider using `unload` method"]
pub struct Module<E: ModuleExportsForHost> {
  pub id: ModuleId,
  pub(crate) library: LeakLibrary,
  library_path: PathBuf,
  pub(crate) internal_exports: InternalModuleExports,
  pub_exports: E,

  /// Module must be loaded and unloaded from the same thread
  _not_thread_safe: PhantomData<*const ()>,
}

impl<E: ModuleExportsForHost> Module<E> {
  pub(crate) fn new(
    id: ModuleId,
    library: Library,
    internal_exports: InternalModuleExports,
    pub_exports: E,
    library_path: PathBuf,
  ) -> Self {
    Self {
      id,
      library: LeakLibrary::new(library),
      library_path,
      internal_exports,
      pub_exports,
      _not_thread_safe: PhantomData,
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
  ///
  /// # Panics
  /// If main function is not exported from the module.
  #[must_use = "returns `None` if module panics"]
  pub unsafe fn call_main<R>(&self) -> Option<ModuleValue<'_, R>> {
    call_module_pub_export(self.library(), "__relib__main").unwrap_or_else(|e| {
      panic!("Failed to get main fn from module, reason: {e:#}");
    })
  }

  /// Unloads module, if it fails, module may be leaked and never be unloaded.
  pub fn unload(self) -> Result<(), UnloadError> {
    let library = self.library();
    let library_path = self.library_path.to_string_lossy().into_owned();

    unsafe {
      // println!("Trying to call \"before_unload\");

      let result = call_module_pub_export(library, "__relib__before_unload");
      match result {
        Ok(Some(())) => {}
        Err(_) => {
          // couldn't get it? it doesn't matter, moving on
        }
        Ok(None) => {
          return Err(UnloadError::BeforeUnloadPanicked(library_path));
        }
      }
    }

    #[cfg(target_os = "linux")]
    {
      let spawned_threads = unsafe { *self.internal_exports.spawned_threads_count() };

      if spawned_threads > 0 {
        return Err(UnloadError::ThreadsStillRunning(library_path));
      }

      unsafe {
        self.internal_exports.lock_module_allocator();
        self.internal_exports.run_thread_local_dtors();
      }
    }

    #[cfg(target_os = "windows")]
    unsafe {
      self.internal_exports.lock_module_allocator();
    }

    module_allocs::remove_module(&self);

    self.library.take().close()?;

    #[cfg(target_os = "linux")]
    {
      use crate::helpers::linux::is_library_loaded;

      let still_loaded = is_library_loaded(&self.library_path);
      if still_loaded {
        return Err(UnloadError::UnloadingFail(library_path));
      }
    }

    Ok(())
  }
}

impl<E: ModuleExportsForHost> Debug for Module<E> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let id = self.id;
    write!(f, "Module {{ id: {id} }}")
  }
}
