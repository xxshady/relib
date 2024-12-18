use thiserror::Error;

#[derive(Error, Debug)]
pub enum LoadError {
  #[error("libloading error")]
  Libloading(#[from] libloading::Error),

  #[error("this module is already loaded")]
  ModuleAlreadyLoaded,

  #[error(
    "module is compiled with different rustc version:\n\
    {0}\n\
    expected:\n\
    {1}"
  )]
  ModuleCompilationMismatch(String, String),
}

#[derive(Error, Debug)]
pub enum UnloadError {
  #[error(
    "module export \"before_unload\" panicked\n\
    module path: {0}"
  )]
  BeforeUnloadPanicked(String),

  #[error(
    "module still has running threads\n\
    module path: {0}\n\
    note: module can export \"before_unload\" function to join spawned threads: \
    https://docs.rs/relib/latest/relib/docs/index.html#before_unload"
  )]
  ThreadsStillRunning(String),

  #[error(
    "libloading unload error\n\
    module path: {0}"
  )]
  Libloading(#[from] libloading::Error),

  #[error(
    "unloading failed for unknown reason (called destructors of thread-locals, checked running threads but it still failed)\n\
    module path: {0}"
  )]
  UnloadingFail(String),
}
