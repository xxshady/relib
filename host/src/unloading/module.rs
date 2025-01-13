use crate::{helpers::call_module_pub_export, Module, ModuleExportsForHost};
use super::{errors::UnloadError, module_allocs};

impl<E: ModuleExportsForHost> Module<E> {
  /// Unloads module, if it fails, module may be leaked and never be unloaded.
  pub fn unload(self) -> Result<(), UnloadError> {
    let library = self.library();
    let library_path = self.library_path.to_string_lossy().into_owned();

    unsafe {
      // println!("Trying to call \"before_unload\");

      let result = call_module_pub_export(library, "before_unload");
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
      let spawned_threads = unsafe { self.internal_exports.spawned_threads_count() };

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
