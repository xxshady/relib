use proc_macro::TokenStream;

/// Define function as exported to the host, currently it's only used for `main` and [`before_unload`](https://docs.rs/relib/latest/relib/docs/index.html#before_unload) functions.
///
/// See
/// [`examples/export_main_macro`](https://github.com/xxshady/relib/blob/main/examples/README.md#customized-relib_moduleexport-proc-macro)
/// for an example how you can customize this proc-macro.
///
/// # Implementation details
/// See documentation of [`relib_exportify::exportify`](https://docs.rs/relib_exportify/latest/relib_exportify/fn.exportify.html).
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
///   module.call_main().map(|v| *v) // call_main returns `Option<ModuleValue<'_, T>>`
/// };
/// dbg!(returned_value);
/// ```
#[proc_macro_attribute]
pub fn export(_args: TokenStream, input: TokenStream) -> TokenStream {
  relib_exportify::exportify(input.into()).into()
}
