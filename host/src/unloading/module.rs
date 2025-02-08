use crate::{
  helpers::{call_module_pub_export, is_library_loaded},
  Module, ModuleExportsForHost,
};
use super::errors::UnloadError;

impl<E: ModuleExportsForHost> Module<E> {
  /// Unloads module, if it fails, module may be leaked and never be unloaded.
  pub fn unload(
    #[allow(unused_mut)] // only used on windows
    mut self,
  ) -> Result<(), UnloadError> {
    let library = self.library();
    let library_path = self.library_path.to_string_lossy().into_owned();

    // calling before_unload callback

    unsafe {
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
      // running threads check (currently its only implemented on linux)
      let spawned_threads = unsafe { self.internal_exports.spawned_threads_count() };
      if spawned_threads > 0 {
        return Err(UnloadError::ThreadsStillRunning(library_path));
      }

      unsafe {
        self.internal_exports.lock_module_allocator();
        self.internal_exports.run_thread_local_dtors();
        self.internal_exports.misc_cleanup();
      }
    }

    // removing module from global allocations store
    // (removing happens later on windows because thread-local destructors
    // are called by standard library in `library.close()`)

    #[cfg(target_os = "linux")]
    super::module_allocs::remove_module(self.id, &self.internal_exports, &library_path);

    #[cfg(target_os = "linux")]
    self.library.take().close()?;

    #[cfg(target_os = "windows")]
    {
      use libloading::os::windows::Library as WindowsLibrary;
      use crate::{windows::dbghelp, unloading::windows_dealloc, leak_library::LeakLibrary};

      let library = self.library.take();
      let handle = WindowsLibrary::from(library).into_raw();

      // re-initializing self.library because windows_dealloc::set
      // takes module instance by value
      // (shouldn't be expensive, just looks weird)
      let library = unsafe { WindowsLibrary::from_raw(handle) };
      let library = libloading::Library::from(library);
      self.library = LeakLibrary::new(library);

      windows_dealloc::set(self, library_path.clone());

      dbghelp::remove_module(handle, &library_path);

      let library = unsafe { WindowsLibrary::from_raw(handle) };
      library.close()?;

      assert!(
        windows_dealloc::successfully_called(),
        "windows dealloc callback must be called in library.close()"
      );
    }

    // final unload check

    let still_loaded = is_library_loaded(&library_path);
    if still_loaded {
      return Err(UnloadError::UnloadingFail(library_path));
    }

    Ok(())
  }
}
