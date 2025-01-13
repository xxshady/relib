use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::ItemFn;

use relib_internal_shared::{fn_inputs_without_types, output_to_return_type};

/// Takes function code and transforms it into exported `extern "C"` function with panic handling.
/// See `relib_export` for proc-macro.
///
/// # Example
/// ```
/// // input:
/// fn foo() -> i32 {
///   // ...
/// }
///
/// // output:
/// #[unsafe(export_name = "...")]
/// extern "C" fn foo(
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
  let mangled_name_ident = format_ident!("{mangled_name}");
  let post_mangled_name_ident = format_ident!("__post{mangled_name}");
  let return_type = output_to_return_type!(output);
  let inputs_without_types = fn_inputs_without_types!(inputs);

  let ret_needs_box = relib_internal_shared::type_needs_box(&return_type.to_string());

  let (return_type, return_value, post_export) = if ret_needs_box {
    (
      quote! { *mut #return_type },
      quote! {
        unsafe {
          use std::boxed::Box;

          let ptr = Box::into_raw(Box::new(return_value));
          ptr
        }
      },
      quote! {
        #[unsafe(no_mangle)]
        pub extern "C" fn #post_mangled_name_ident(
          return_value_ptr: *mut #return_type
        ) {
          use std::boxed::Box;
          unsafe {
            drop(Box::from_raw(return_value_ptr));
          }
        }
      },
    )
  } else {
    (
      return_type,
      quote! {
        return_value
      },
      quote! {},
    )
  };

  quote! {
    #[unsafe(export_name = #mangled_name)]
    #( #attrs )*
    pub unsafe extern "C" fn #ident(
      ____success____: *mut bool,
      #inputs
    ) -> std::mem::MaybeUninit<#return_type> // will be initialized if function won't panic
    {
      fn #mangled_name_ident( #inputs ) #output #block

      let result = std::panic::catch_unwind(|| {
        #mangled_name_ident( #( #inputs_without_types )* )
      });
      match result {
        Ok(return_value) => {
          unsafe {
            *____success____ = true;
          }

          #[allow(unused_braces)]
          std::mem::MaybeUninit::new({ #return_value })
        }
        // ignoring content since it's handled in our panic hook
        Err(_) => {
          unsafe {
            *____success____ = false;
          }

          std::mem::MaybeUninit::uninit()
        }
      }
    }

    #post_export
  }
}
