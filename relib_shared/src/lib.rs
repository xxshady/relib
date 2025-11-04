use std::{
  alloc::Layout,
  collections::{BTreeMap, BTreeSet, HashMap, HashSet, LinkedList, VecDeque},
  fmt::Debug,
};

pub type ModuleId = u64;

// TODO: improve this description:
/// A trait for moving allocations to the host (executable) from the module (cdylib).
/// This is necessary since module has global alloc tracker allocations,
/// when module is unloaded all leaks are deallocated manually so we need to manually
/// remove moved allocations from the global alloc tracker.
#[diagnostic::on_unimplemented(
  message = "`{Self}` cannot be moved between the host and modules",
  note = "see relib docs: TODO: link"
)]
pub unsafe trait Transfer<F>
where
  // TODO: turn it into associated type?
  F: TransferTarget,
{
  /// # Safety
  /// After calling this on a container it's your responsibility to immediately move
  /// it to the host or module.
  unsafe fn transfer(&self, ctx: &F::ExtraContext);
}

pub unsafe trait TransferTarget {
  type ExtraContext: Debug;
  fn transfer(ptr: *mut u8, layout: Layout, ctx: &Self::ExtraContext);
}

macro_rules! impl_for_copy {
  (
    $(
      $ty:ty $(, generics: $($generics:ident),+)?
    )+
  ) => {
    $(
      unsafe impl<
        F: TransferTarget
        $(, $($generics),+ )?
      >
      Transfer<F> for $ty
      where
        F: TransferTarget,
      {
        unsafe fn transfer(&self, _ctx: &F::ExtraContext) {
          // this type is copy so there is no allocation to transfer
        }
      }
    )+
  };
}

impl_for_copy!(
  () u8 i8 u16 i16 u32 i32 u64 i64 u128 i128 usize isize
  f32 f64 bool char

  &str // TODO: is it safe? &'static str?

  *mut T, generics: T
  *const T, generics: T

  // TODO: more impls
  fn(T1) -> T2, generics: T1, T2
  fn(T1, T2) -> T3, generics: T1, T2, T3
  unsafe fn(T1) -> T2, generics: T1, T2
  unsafe fn(T1, T2) -> T3, generics: T1, T2, T3
);

unsafe impl<T, F> Transfer<F> for Vec<T>
where
  T: Transfer<F>,
  F: TransferTarget,
{
  unsafe fn transfer(&self, ctx: &F::ExtraContext) {
    let ptr = self.as_ptr();
    println!("[transfer] Vec {ptr:?} ctx: {ctx:?}");

    F::transfer(ptr as *mut u8, ctx);

    for el in self {
      unsafe {
        Transfer::<F>::transfer(el, ctx);
      }
    }
  }
}

unsafe impl<T, F> Transfer<F> for Box<T>
where
  T: Transfer<F>,
  F: TransferTarget,
{
  unsafe fn transfer(&self, ctx: &F::ExtraContext) {
    let ptr = &**self as *const T;
    println!("[transfer] Box {ptr:?} ctx: {ctx:?}");

    F::transfer(ptr as *mut u8, ctx);
    // The inner value also needs to be transferred if it contains allocations
    unsafe {
      Transfer::<F>::transfer(&**self, ctx);
    }
  }
}

unsafe impl<F: TransferTarget> Transfer<F> for String {
  unsafe fn transfer(&self, ctx: &F::ExtraContext) {
    let ptr = self.as_ptr();
    println!("[transfer] String {ptr:?} ctx: {ctx:?}");
    F::transfer(ptr as *mut u8, ctx);
  }
}

unsafe impl<T, F> Transfer<F> for Option<T>
where
  T: Transfer<F>,
  F: TransferTarget,
{
  unsafe fn transfer(&self, ctx: &F::ExtraContext) {
    if let Some(inner) = self {
      unsafe { Transfer::<F>::transfer(inner, ctx) };
    }
  }
}

unsafe impl<T, E, F> Transfer<F> for Result<T, E>
where
  T: Transfer<F>,
  E: Transfer<F>,
  F: TransferTarget,
{
  unsafe fn transfer(&self, ctx: &F::ExtraContext) {
    unsafe {
      match self {
        Ok(inner) => Transfer::<F>::transfer(inner, ctx),
        Err(inner) => Transfer::<F>::transfer(inner, ctx),
      }
    }
  }
}

unsafe impl<K, V, F> Transfer<F> for HashMap<K, V>
where
  K: Transfer<F>,
  V: Transfer<F>,
  F: TransferTarget,
{
  unsafe fn transfer(&self, ctx: &F::ExtraContext) {
    for (k, v) in self {
      unsafe {
        Transfer::<F>::transfer(k, ctx);
        Transfer::<F>::transfer(v, ctx);
      }
    }
  }
}

unsafe impl<K, V, F> Transfer<F> for BTreeMap<K, V>
where
  K: Transfer<F>,
  V: Transfer<F>,
  F: TransferTarget,
{
  unsafe fn transfer(&self, ctx: &F::ExtraContext) {
    for (k, v) in self {
      unsafe {
        Transfer::<F>::transfer(k, ctx);
        Transfer::<F>::transfer(v, ctx);
      }
    }
  }
}

unsafe impl<T, F> Transfer<F> for HashSet<T>
where
  T: Transfer<F>,
  F: TransferTarget,
{
  unsafe fn transfer(&self, ctx: &F::ExtraContext) {
    for el in self {
      unsafe {
        Transfer::<F>::transfer(el, ctx);
      }
    }
  }
}

unsafe impl<T, F> Transfer<F> for BTreeSet<T>
where
  T: Transfer<F>,
  F: TransferTarget,
{
  unsafe fn transfer(&self, ctx: &F::ExtraContext) {
    for el in self {
      unsafe {
        Transfer::<F>::transfer(el, ctx);
      }
    }
  }
}

unsafe impl<T, F> Transfer<F> for LinkedList<T>
where
  T: Transfer<F>,
  F: TransferTarget,
{
  unsafe fn transfer(&self, ctx: &F::ExtraContext) {
    for el in self {
      unsafe {
        Transfer::<F>::transfer(el, ctx);
      }
    }
  }
}

unsafe impl<T, F> Transfer<F> for VecDeque<T>
where
  T: Transfer<F>,
  F: TransferTarget,
{
  unsafe fn transfer(&self, ctx: &F::ExtraContext) {
    for el in self {
      unsafe {
        Transfer::<F>::transfer(el, ctx);
      }
    }
  }
}

unsafe impl<T, F, const N: usize> Transfer<F> for [T; N]
where
  T: Transfer<F>,
  F: TransferTarget,
{
  unsafe fn transfer(&self, ctx: &F::ExtraContext) {
    for el in self {
      unsafe {
        Transfer::<F>::transfer(el, ctx);
      }
    }
  }
}

macro_rules! impl_for_tuples {
  (
    $(
      ( $( $T:ident ),+ );
    )+
  ) => {
    $(
      unsafe impl<F, $( $T, )+> Transfer<F> for ( $( $T, )+ )
      where
        F: TransferTarget,
        $( $T: Transfer<F>, )+
      {
        unsafe fn transfer(&self, ctx: &F::ExtraContext) {
          #[allow(non_snake_case)]
          let ( $( $T, )+ ) = self;

          unsafe {
            $( Transfer::<F>::transfer($T, ctx); )+
          }
        }
      }
    )+
  };
}

impl_for_tuples!(
  (T1);
  (T1, T2);
  (T1, T2, T3);
  (T1, T2, T3, T4);
  (T1, T2, T3, T4, T5);
  (T1, T2, T3, T4, T5, T6);
  (T1, T2, T3, T4, T5, T6, T7);
  (T1, T2, T3, T4, T5, T6, T7, T8);
  (T1, T2, T3, T4, T5, T6, T7, T8, T9);
  (T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);
  (T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11);
  (T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12);
  (T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13);
  (T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14);
  (T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15);
  (T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16);
);
