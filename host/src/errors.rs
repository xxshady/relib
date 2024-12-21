use thiserror::Error;

#[derive(Error, Debug)]
pub enum LoadError {
  #[error("libloading error")]
  Libloading(#[from] libloading::Error),

  #[error("this module is already loaded")]
  ModuleAlreadyLoaded,

  #[error(
    "module is compiled with different configuration:\n\
    {module}\n\
    expected:\n\
    {host}\n\
    note: make sure that host and module are compiled with identical rustc version,\n\
    relib version (relib_module and relib_host dependency versions must be identical)\n\
    and with identical relib features: \"unloading\" enabled/disabled"
  )]
  ModuleCompilationMismatch { module: String, host: String },
}
