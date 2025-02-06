use crate::{
  helpers::{call_module_pub_export, is_library_loaded, path_to_str},
  unloading::helpers::unrecoverable,
  windows, Module, ModuleExportsForHost,
};
use super::{errors::UnloadError, module_allocs};

impl<E: ModuleExportsForHost> Module<E> {
  /// Unloads module, if it fails, module may be leaked and never be unloaded.
  pub fn unload(self) -> Result<(), UnloadError> {
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
        self.internal_exports.unmap_all_mmaps();
      }
    }

    // removing module from global allocations store
    // (removing happens later on windows because thread-local destructors
    // are called by standard library in `library.close()`)

    #[cfg(target_os = "linux")]
    module_allocs::remove_module(self.id, &self.internal_exports, &library_path);

    let library_path_str = library_path.as_str();

    #[cfg(target_os = "linux")]
    self.library.take().close()?;

    #[cfg(target_os = "windows")]
    {
      use std::{cell::RefCell, ffi::c_void};
      use libloading::os::windows::Library as WindowsLibrary;

      let library = self.library.take();

      // converting it into raw handle
      let handle = WindowsLibrary::from(library).into_raw();

      thread_local! {
        static DEALLOC_CLOSURE: RefCell<Option<Box<dyn FnOnce()>>> = Default::default();
      }

      extern "C" fn dealloc_callback() {
        DEALLOC_CLOSURE.with_borrow_mut(|v| {
          let callback = v.take().unwrap_or_else(|| {
            unrecoverable("DEALLOC_CLOSURE must be set when dealloc_callback is called");
          });
          callback();
        });
      }

      unsafe {
        self
          .internal_exports
          .set_dealloc_callback(dealloc_callback as *const c_void);
      }

      DEALLOC_CLOSURE.set(Some(Box::new({
        let library_path = library_path.clone();

        move || {
          unsafe {
            self.internal_exports.lock_module_allocator();
          }
          windows::dbghelp::remove_module(handle, &library_path);
          module_allocs::remove_module(self.id, &self.internal_exports, &library_path);
        }
      })));

      // converting it back into library instance
      let library = unsafe { WindowsLibrary::from_raw(handle) };
      library.close()?;

      assert!(
        DEALLOC_CLOSURE.take().is_none(),
        "DEALLOC_CLOSURE must be called in library.close()"
      );
    }

    // final unload check

    let still_loaded = is_library_loaded(library_path_str);
    if still_loaded {
      return Err(UnloadError::UnloadingFail(library_path));
    }

    Ok(())
  }
}
