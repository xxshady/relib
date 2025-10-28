use perfect_api::ApiState;

#[repr(C)]
#[derive(Debug)]
pub struct State {
  pub counter: u32,
  pub perfect_api_state: ApiState,
  pub entity: perfect_api::Entity,

  pub vec: SVec<u8>,
}

// ABI-stable Vec
#[repr(C)]
#[derive(Debug)]
pub struct SVec<T> {
  ptr: *mut T,
  len: usize,
  cap: usize,
}

impl<T> SVec<T> {
  pub fn into_vec(self) -> Vec<T> {
    self.into()
  }
}

impl<T> Drop for SVec<T> {
  fn drop(&mut self) {
    let ptr = self.ptr;
    let len = self.len;
    let cap = self.cap;

    drop(vec_from_svec(ptr, len, cap))
  }
}

impl<T> Default for SVec<T> {
  fn default() -> Self {
    Vec::default().into()
  }
}

impl<T> From<Vec<T>> for SVec<T> {
  fn from(mut value: Vec<T>) -> Self {
    let ptr = value.as_mut_ptr();
    let len = value.len();
    let cap = value.capacity();

    // moving ownership to SVec
    std::mem::forget(value);

    Self { ptr, len, cap }
  }
}

impl<T> From<SVec<T>> for Vec<T> {
  fn from(value: SVec<T>) -> Self {
    let ptr = value.ptr;
    let len = value.len;
    let cap = value.cap;

    std::mem::forget(value);

    vec_from_svec(ptr, len, cap)
  }
}

fn vec_from_svec<T>(ptr: *mut T, len: usize, cap: usize) -> Vec<T> {
  // SAFETY: should be safe since we only create SVec from normal Vec
  unsafe { Vec::from_raw_parts(ptr, len, cap) }
}
