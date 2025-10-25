// perhaps hot reload will be implemented directly in relib in the future
pub use relib_internal_shared::StableLayout;

pub mod exports;
pub mod shared_imports;

pub const EMPTY_EXPORTS: &str = include_str!("exports.rs");
pub const SHARED_IMPORTS: &str = include_str!("shared_imports.rs");

pub type Alloc = unsafe extern "C" fn(layout: StableLayout) -> *mut u8;
pub type Dealloc = unsafe extern "C" fn(ptr: *mut u8, layout: StableLayout);

#[repr(C)]
#[derive(Clone, Copy)]
pub struct MainModuleRet {
  // state is opaque pointer here because it's owned by main module allocator
  // (it will deallocate it at unloading) and host should not mutate it.
  // also passing it as opaque pointer allows to change it's layout between live reloads (but not hot reloads ofc)
  pub state: *mut (),
  pub alloc: Alloc,
  pub dealloc: Dealloc,
}
