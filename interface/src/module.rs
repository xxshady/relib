use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::FnArg;

use relib_internal_shared::output_to_return_type;

use crate::shared::{
  extract_trait_name_from_path, for_each_trait_item, parse_trait_file, type_needs_box,
  write_code_to_file, TraitFn, SAFETY_DOC,
};

/// Will generate `generated_module_exports.rs` and `generated_module_imports.rs` in the OUT_DIR which you can include
/// using `include!(concat!(env!("OUT_DIR"), "/<file>"));` in your `lib.rs` or `main.rs`
/// and then use `ModuleExportsImpl` struct to implement your `Exports` trait:
/// ```
/// impl Exports for ModuleExportsImpl {
///   // ...
/// }
/// ```
#[cfg(feature = "internal")]
pub fn generate_internal(
  exports_file_content: &'static str,
  exports_trait_path: &str,
  imports_file_content: &'static str,
  imports_trait_path: &str,
) {
  generate_exports(exports_file_content, exports_trait_path, false);
  generate_imports(imports_file_content, imports_trait_path, false);
}

/// Will generate `generated_module_exports.rs` and `generated_module_imports.rs` in the OUT_DIR which you can include
/// using `include_exports!();` and `include_imports!();` in your `lib.rs` or `main.rs`
/// and then use `ModuleExportsImpl` struct to implement your `Exports` trait:
/// ```
/// impl Exports for ModuleExportsImpl {
///   // ...
/// }
/// ```
#[cfg(feature = "public")]
pub fn generate(
  exports_file_content: &'static str,
  exports_trait_path: &str,
  imports_file_content: &'static str,
  imports_trait_path: &str,
) {
  generate_exports(exports_file_content, exports_trait_path, true);
  generate_imports(imports_file_content, imports_trait_path, true);
}

fn generate_exports(
  exports_file_content: &'static str,
  exports_trait_path: &str,
  pub_exports: bool,
) {
  let trait_name = extract_trait_name_from_path(exports_trait_path);

  let (exports_trait, module_use_items) =
    parse_trait_file(trait_name, exports_file_content, exports_trait_path);

  let exports_trait_path: syn::Path =
    syn::parse_str(exports_trait_path).expect("Failed to parse exports_trait_path as syn::Path");

  let mut exports = Vec::<TokenStream2>::new();

  for item in exports_trait.items {
    let TraitFn {
      ident,
      inputs,
      inputs_without_types,
      output,
      mangled_name,
      mangled_ident,
      post_ident: _,
      post_mangled_name: _,
      post_mangled_ident,
      lifetimes_for: _,
      lifetimes_full,
    } = for_each_trait_item(trait_name, &item);

    // !!! keep in sync with main and before_unload calls in relib_host crate !!!
    let code = if pub_exports {
      let return_type = output_to_return_type!(output);
      let needs_box = type_needs_box(&return_type);
      let (return_type, return_value, post_export) = if needs_box {
        (
          quote! {
            *mut #return_type
          },
          quote! {
            use std::boxed::Box;
            Box::into_raw(Box::new(return_value))
          },
          quote! {
            #[unsafe(no_mangle)]
            pub extern "C" fn #post_mangled_ident #lifetimes_full (
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
        (return_type, quote! { return_value }, quote! {})
      };

      quote! {
        #[unsafe(no_mangle)]
        pub extern "C" fn #mangled_ident #lifetimes_full (
          ____success____: *mut bool,
          #inputs
        ) -> std::mem::MaybeUninit<#return_type> // will be initialized if function won't panic
        {
          let result = std::panic::catch_unwind(move || {
            <ModuleExportsImpl as Exports>::#ident( #( #inputs_without_types )* )
          });

          match result {
            Ok(return_value) => {
              unsafe {
                *____success____ = true;
              }

              #[allow(unused_braces)]
              std::mem::MaybeUninit::new({ #return_value })
            }
            // ignoring content since it's handled in our panic hook or
            // in default panic hook of std when unloading is disabled
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
    } else {
      quote! {
        #[unsafe(export_name = #mangled_name)]
        pub extern "C" fn #ident #lifetimes_full ( #inputs ) #output {
          <ModuleExportsImpl as Exports>::#ident( #( #inputs_without_types )* )
        }
      }
    };

    exports.push(code);
  }

  write_code_to_file(
    "generated_module_exports.rs",
    quote! {
      #module_use_items

      use #exports_trait_path as Exports;

      /// Struct for implementing your `Exports` trait
      pub struct ModuleExportsImpl;

      #( #exports )*
    },
  );
}

fn generate_imports(
  imports_file_content: &'static str,
  imports_trait_path: &str,
  pub_imports: bool,
) {
  let trait_name = extract_trait_name_from_path(imports_trait_path);
  let (imports_trait, module_use_items) =
    parse_trait_file(trait_name, imports_file_content, imports_trait_path);

  let mut imports = Vec::<TokenStream2>::new();

  for item in imports_trait.items {
    let TraitFn {
      ident,
      inputs,
      inputs_without_types,
      output,
      mangled_name: _,
      mangled_ident,
      post_ident: _,
      post_mangled_name: _,
      post_mangled_ident,
      lifetimes_for,
      lifetimes_full,
    } = for_each_trait_item(trait_name, &item);

    let placeholder_inputs: TokenStream2 = inputs
      .iter()
      .map(|arg| {
        let FnArg::Typed(arg) = arg else {
          unreachable!();
        };

        let ty = &arg.ty;
        quote! { _: #ty , }
      })
      .collect();

    let function_sig = quote! {
      #[doc = #SAFETY_DOC]
      pub unsafe fn #ident #lifetimes_full ( #inputs ) #output
    };

    let function_static_decl = quote! {
      #[allow(non_upper_case_globals)]
      #[unsafe(no_mangle)]
      static mut #mangled_ident
    };

    let suppress_lints_for_return_value = quote! {
      #[allow(unused_variables, clippy::let_unit_value, clippy::diverging_sub_expression)]
    };

    // !!! keep in sync with main and before_unload calls in relib_host crate !!!
    let function_body = if pub_imports {
      let return_type = output_to_return_type!(output);
      let needs_box = type_needs_box(&return_type);

      let (return_type, post_fn_decl, read_return_value) = if needs_box {
        (
          quote! { *mut #return_type },
          quote! {
            #[allow(non_upper_case_globals)]
            #[unsafe(no_mangle)]
            static mut #post_mangled_ident: #lifetimes_for extern "C" fn(return_value_ptr: *mut #return_type) = ____post_placeholder____;

            extern "C" fn ____post_placeholder____ #lifetimes_full (_: *mut #return_type) {
              unreachable!();
            }
          },
          quote! {
            let return_ptr = return_value.assume_init();
            let return_value: #return_type = unsafe {
              Clone::clone(&*return_ptr)
            };
            #post_mangled_ident(return_ptr);
            return_value
          },
        )
      } else {
        (
          return_type,
          quote! {},
          quote! {
            return_value.assume_init()
          },
        )
      };

      quote! {
        #function_static_decl: #lifetimes_for extern "C" fn(
          ____success____: *mut bool,
          #inputs
        ) -> std::mem::MaybeUninit<#return_type> = ____placeholder____;

        extern "C" fn ____placeholder____ #lifetimes_full (
          _: *mut bool,
          #placeholder_inputs
        ) -> std::mem::MaybeUninit<#return_type> {
          unreachable!();
        }

        #post_fn_decl

        let mut ____success____ = std::mem::MaybeUninit::<bool>::uninit();

        #suppress_lints_for_return_value
        let return_value = unsafe {
          #mangled_ident( ____success____.as_mut_ptr(), #( #inputs_without_types )* )
        };

        if !____success____.assume_init() {
          // TODO: expose unrecoverable helper in relib_module::__internal and use it here?
          eprintln!("[relib] host panicked while executing import {:?} of module, aborting", stringify!(#ident));
          std::process::abort();
        }

        // SAFETY: function returned true so we are allowed to read the return value
        #read_return_value
      }
    } else {
      quote! {
        #function_static_decl: #lifetimes_for extern "C" fn( #inputs ) #output = placeholder;

        extern "C" fn placeholder #lifetimes_full ( #placeholder_inputs ) #output {
          unreachable!();
        }

        unsafe {
          #mangled_ident( #( #inputs_without_types )* )
        }
      }
    };

    let full_function = quote! {
      #function_sig {
        #function_body
      }
    };

    imports.push(full_function);
  }

  write_code_to_file(
    "generated_module_imports.rs",
    quote! {
      #module_use_items

      #( #imports )*
    },
  );
}
