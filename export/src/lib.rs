use proc_macro::TokenStream;

/// Use it for exporting `main` or `before_unload` functions from the module.
///
/// **note:** see documentation of [`relib_exportify::exportify`](https://docs.rs/relib_exportify/latest/relib_exportify/fn.exportify.html) for implementation details.
///
/// # Examples
/// ```
/// #[relib_module::export]
/// fn main() -> bool {
///   // ...
///   true
/// }
///
/// // on host side
/// let returned_value: Option<bool> = unsafe {
///   module.call_main().map(|v| *v)
/// };
/// dbg!(returned_value);
/// ```
#[proc_macro_attribute]
pub fn export(_args: TokenStream, input: TokenStream) -> TokenStream {
  relib_exportify::exportify(input.into()).into()
}
