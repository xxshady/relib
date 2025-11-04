use {
  relib_derive::Transfer,
  std::{
    alloc::Layout,
    fmt::{Debug, Formatter, Result as FmtResult},
  },
};

pub mod exports;
pub mod imports;
pub mod exports_no_unloading;

pub const EXPORTS: &str = include_str!("exports.rs");
pub const IMPORTS: &str = include_str!("imports.rs");
pub const EXPORTS_NO_UNLOADING: &str = include_str!("exports_no_unloading.rs");

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
// TODO: remove this type?
#[derive(Transfer)]
pub struct AllocatorPtr(pub *mut u8);

// SAFETY: `*mut u8` won't be touched anywhere except in the dynamic library in the main thread for deallocation
unsafe impl Send for AllocatorPtr {}
unsafe impl Sync for AllocatorPtr {}

#[repr(C)]
#[derive(Clone, PartialEq)]
// TODO: remove this type?
#[derive(Transfer)]
pub struct Allocation(pub AllocatorPtr, pub StableLayout);

impl Debug for Allocation {
  fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
    let Self(AllocatorPtr(ptr), StableLayout { size, align }) = self;
    write!(f, "({ptr:?}, size: {size:?} align: {align:?})")
  }
}

#[repr(C)]
#[derive(
  Clone,
  Copy,
  PartialEq,
  Debug,
  // TODO: remove this type?
  Transfer,
)]
pub struct StableLayout {
  size: usize,
  align: usize,
}

impl From<Layout> for StableLayout {
  fn from(layout: Layout) -> Self {
    Self {
      size: layout.size(),
      align: layout.align(),
    }
  }
}

impl From<StableLayout> for Layout {
  fn from(value: StableLayout) -> Self {
    // SAFETY: StableLayout can only be created from valid Layout (see From<Layout> impl)
    unsafe { Layout::from_size_align_unchecked(value.size, value.align) }
  }
}

impl From<&StableLayout> for Layout {
  fn from(value: &StableLayout) -> Self {
    Layout::from(*value)
  }
}

#[repr(C)]
#[derive(Clone, PartialEq, Debug)]
// TODO: remove this type?
#[derive(Transfer)]
pub enum AllocatorOp {
  Alloc(Allocation),
  Dealloc(Allocation),
}

pub type SliceAllocatorOp = RawSlice<AllocatorOp>;
pub type SliceAllocation = RawSlice<Allocation>;

/// ABI-stable `&[T]`
#[repr(C)]
// TODO: remove this type?
#[derive(Transfer)]
pub struct RawSlice<T> {
  pub ptr: *const T,
  pub len: usize,
}

impl<T> RawSlice<T> {
  /// # Safety
  /// See `Safety` of [`std::slice::from_raw_parts`]
  pub unsafe fn into_slice<'a>(self) -> &'a [T] {
    unsafe { std::slice::from_raw_parts(self.ptr, self.len) }
  }

  /// # Safety
  /// See `Safety` of [`std::slice::from_raw_parts`]
  pub unsafe fn to_vec(&self) -> Vec<T>
  where
    T: Clone,
  {
    unsafe { std::slice::from_raw_parts(self.ptr, self.len).to_vec() }
  }
}

impl<T> From<&[T]> for RawSlice<T> {
  fn from(value: &[T]) -> Self {
    RawSlice {
      ptr: value.as_ptr(),
      len: value.len(),
    }
  }
}

/// ABI-stable `&str`
#[repr(C)]
// TODO: remove this type?
#[derive(Transfer)]
pub struct Str(RawSlice<u8>);

impl Str {
  /// # Safety
  /// See `Safety` of [`std::slice::from_raw_parts`]
  pub unsafe fn into_str<'a>(self) -> &'a str {
    let bytes = unsafe { self.0.into_slice() };
    std::str::from_utf8(bytes).expect("Failed to get valid UTF-8 string slice back")
  }

  /// # Safety
  /// See `Safety` of [`std::slice::from_raw_parts`]
  pub unsafe fn to_string(&self) -> String {
    let bytes = unsafe { self.0.to_vec() };
    String::from_utf8(bytes).expect("Failed to convert to valid UTF-8 string")
  }

  /// `From<&str>` for const contexts
  pub const fn const_from(value: &str) -> Self {
    let bytes = value.as_bytes();
    Self(RawSlice {
      ptr: bytes.as_ptr(),
      len: bytes.len(),
    })
  }
}

impl From<&str> for Str {
  fn from(value: &str) -> Self {
    Self(value.as_bytes().into())
  }
}

// SAFETY: `&str` is Send and Sync
unsafe impl Send for Str {}
unsafe impl Sync for Str {}

#[macro_export]
macro_rules! output_to_return_type {
  ($output:ident) => {
    match &$output {
      syn::ReturnType::Default => {
        quote! { () }
      }
      syn::ReturnType::Type(_, ty) => quote::ToTokens::to_token_stream(ty),
    }
  };
}

#[macro_export]
macro_rules! fn_inputs_without_types {
  ($inputs:expr) => {
    $inputs
      .iter()
      .map(|arg| {
        let syn::FnArg::Typed(arg) = arg else {
          unreachable!();
        };

        quote::ToTokens::to_token_stream(&arg.pat)
      })
      .collect::<Vec<_>>()
  };
}

// TODO: remove it?
pub fn type_needs_box(_type_: &str) -> bool {
  false

  // let stable_copy_type = [
  //   "()", "bool", "u8", "i8", "u16", "i16", "u32", "i32", "u64", "i64", "usize", "isize", "f32",
  //   "f64", "char", "u128", "i128",
  // ]
  // .contains(&type_)
  //   || type_.starts_with(['*', '&']); // a pointer or a reference

  // !stable_copy_type
}

pub type Alloc = unsafe fn(layout: StableLayout) -> *mut u8;
pub type Dealloc = unsafe fn(ptr: *mut u8, layout: StableLayout);
