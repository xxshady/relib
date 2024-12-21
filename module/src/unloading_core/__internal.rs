/// Used to inform AllocTracker about moved allocations from the host.
/// ```
/// // host:
/// // let's pretend that Vec<u8> is ffi-safe
/// impl Imports for ModuleImportsImpl {
///   fn foo() -> Vec<u8> {
///     vec![1, 2, 3]
///   }
/// }
///
/// // module:
/// // `foo` will call `drop_immediately_and_clone`
/// let vec = unsafe { gen_imports::foo() };
///
/// // now we can safely leak it (it will be deallocated on module unload)
/// // because AllocTracker knows about this allocation
/// std::mem::forget(vec);
/// ```
pub fn drop_immediately_and_clone<T: Clone>(owned_by_host: T) -> T {
  owned_by_host.clone()
}
