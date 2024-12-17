use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{Ident, ItemFn};

use relib_internal_shared::{fn_inputs_without_types, output_to_return_type};

/// Takes function code and transforms it into exported `extern "C"` function with panic handling.
/// See `relib_export` for proc-macro.
///
/// ```
/// // input:
/// fn foo() -> i32 {
///   // ...
/// }
///
/// // output:
/// #[unsafe(export_name = "...")]
/// fn foo(
///   ____return_value____: *mut std::mem::MaybeUninit<i32>,
/// ) -> bool {
///   fn ____wrapper____() -> i32 {
///     // ...
///   }
///
///   let result = std::panic::catch_unwind(____wrapper____);
///   // ...  
/// }
/// ```
pub fn exportify(input: TokenStream2) -> TokenStream2 {
  let input = syn::parse2(input);

  let input: ItemFn = match input {
    Ok(input) => input,
    Err(e) => return e.to_compile_error(),
  };

  let ItemFn {
    attrs,
    vis: _,
    sig,
    block,
  } = input;

  let output = sig.output;
  let inputs = sig.inputs;
  let ident = sig.ident;
  let mangled_name = format!("__relib__{ident}");
  let mangled_name_ident = Ident::new(&mangled_name, Span::call_site());
  let return_type = output_to_return_type!(output);
  let inputs_without_types = fn_inputs_without_types!(inputs);

  quote! {
    #[unsafe(export_name = #mangled_name)]
    #( #attrs )*
    extern "C" fn #ident(
      ____return_value____: *mut std::mem::MaybeUninit<#return_type>, // will be initialized if function won't panic
      #inputs
    ) -> bool // returns false if function panicked
    {
      fn #mangled_name_ident( #inputs_without_types ) #output #block

      let result = std::panic::catch_unwind(#mangled_name_ident);
      match result {
        Ok(value) => {
          unsafe {
            (*____return_value____).write(value);
          }
          true
        }
        // ignoring content since it's handled in our panic hook
        Err(_) => { false }
      }
    }
  }
}
