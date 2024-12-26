use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{FnArg, Ident};

use relib_internal_shared::output_to_return_type;

use crate::shared::{
  extract_trait_name_from_path, for_each_trait_item, parse_trait_file, write_code_to_file, TraitFn,
  SAFETY_DOC,
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
    } = for_each_trait_item(trait_name, &item);

    let mangled_name = Ident::new(&mangled_name, Span::call_site());

    let code = if pub_exports {
      let return_type = output_to_return_type!(output);

      quote! {
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn #mangled_name(
          ____return_value____: *mut std::mem::MaybeUninit<#return_type>, // will be initialized if function won't panic
          #inputs
        ) -> bool // returns false if function panicked
        {
          let result = std::panic::catch_unwind(move || {
            <ModuleExportsImpl as Exports>::#ident( #inputs_without_types )
          });

          match result {
            Ok(value) => {
              (*____return_value____).write(value);
              true
            }
            // ignoring content since it's handled in our panic hook or
            // in default panic hook of std when unloading is disabled
            Err(_) => { false }
          }
        }
      }
    } else {
      quote! {
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn #mangled_name( #inputs ) #output {
          <ModuleExportsImpl as Exports>::#ident( #inputs_without_types )
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

  #[cfg(feature = "unloading")]
  let internals_crate = if pub_imports {
    quote! { relib_module }
  } else {
    quote! { crate }
  };

  let mut imports = Vec::<TokenStream2>::new();

  for item in imports_trait.items {
    let TraitFn {
      ident,
      inputs,
      inputs_without_types,
      output,
      mangled_name,
    } = for_each_trait_item(trait_name, &item);

    let mangled_name = Ident::new(&mangled_name, Span::call_site());

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

    let return_type = output_to_return_type!(output);

    let function_sig = quote! {
      #[doc = #SAFETY_DOC]
      pub unsafe fn #ident( #inputs ) #output
    };

    let function_static_decl = quote! {
      #[allow(non_upper_case_globals)]
      #[unsafe(no_mangle)]
      static mut #mangled_name
    };

    let suppress_lints_for_return_value = quote! {
      #[allow(unused_variables, clippy::let_unit_value, clippy::diverging_sub_expression)]
    };

    #[cfg(feature = "unloading")]
    let return_handling = quote! {
      #[allow(unreachable_code)]
      #internals_crate::__internal::drop_immediately_and_clone(return_value)
    };
    #[cfg(not(feature = "unloading"))]
    let return_handling = quote! {
      return_value
    };

    let function_body = if pub_imports {
      quote! {
        #function_static_decl: unsafe extern "C" fn(
          ____return_value____: *mut std::mem::MaybeUninit<#return_type>, // will be initialized if function won't panic
          #inputs
        ) -> bool = placeholder;

        unsafe extern "C" fn placeholder(
          _: *mut std::mem::MaybeUninit<#return_type>,
          #inputs
        ) -> bool {
          unreachable!();
        }

        let mut ____return_value____ = std::mem::MaybeUninit::<#return_type>::uninit();

        #suppress_lints_for_return_value
        let success = unsafe {
          #mangled_name( &mut ____return_value____, #inputs_without_types )
        };

        if !success {
          // TODO: expose unrecoverable helper in relib_module::__internal and use it here?
          eprintln!("[relib] host panicked while executing an import of module, aborting");
          std::process::abort();
        }

        // SAFETY: function returned true so we are allowed to read the pointer
        #suppress_lints_for_return_value
        let return_value = unsafe {
          ____return_value____.assume_init_read()
        };
        #return_handling
      }
    } else {
      quote! {
        #function_static_decl: unsafe extern "C" fn( #inputs ) #output = placeholder;

        unsafe extern "C" fn placeholder( #placeholder_inputs ) #output {
          unreachable!();
        }

        #suppress_lints_for_return_value
        let return_value = unsafe {
          #mangled_name( #inputs_without_types )
        };
        #return_handling
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
