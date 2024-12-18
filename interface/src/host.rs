use std::{fmt::Debug, path::Path};

use proc_macro2::TokenStream;
use quote::quote;

use relib_internal_shared::output_to_return_type;

use crate::shared::{
  extract_trait_name_from_path, for_each_trait_item, parse_trait_file, write_code_to_file, TraitFn,
  SAFETY_DOC,
};

/// Will generate `generated_module_exports.rs` and `generated_module_imports.rs` in the OUT_DIR which you can include
/// using `include!(concat!(env!("OUT_DIR"), "/<file>"));` in your `lib.rs` or `main.rs`
/// and then use `ModuleExports` struct:
/// ```
/// let exports = ModuleExports::new(library);
/// exports.something();
/// ```
#[cfg(feature = "internal")]
pub fn generate_internal(
  exports_file_content: &'static str,
  exports_trait_path: &str,
  imports_file_content: &'static str,
  imports_trait_path: &str,
) {
  generate_exports(exports_file_content, exports_trait_path, false);
  generate_imports(imports_file_content, imports_trait_path);
}

/// Will generate `generated_module_exports.rs` and `generated_module_imports.rs` in the OUT_DIR which you can include
/// using `include!(concat!(env!("OUT_DIR"), "/<file>"));` in your `lib.rs` or `main.rs`
/// and then use `ModuleExports` struct:
/// ```
/// let exports = ModuleExports::new(library);
/// exports.something();
/// ```
#[cfg(feature = "public")]
pub fn generate(
  exports_file_content: &'static str,
  exports_trait_path: &str,
  imports_file_content: &'static str,
  imports_trait_path: &str,
) {
  generate_exports(exports_file_content, exports_trait_path, true);
  generate_imports(imports_file_content, imports_trait_path);
}

fn generate_exports(
  exports_file_content: &'static str,
  exports_trait_path: &str,
  pub_exports: bool,
) {
  let trait_name = extract_trait_name_from_path(exports_trait_path);
  let (exports_trait, module_use_items) =
    parse_trait_file(trait_name, exports_file_content, exports_trait_path);

  let mut export_decls = Vec::<TokenStream>::new();
  let mut export_inits = Vec::<TokenStream>::new();
  let mut export_impls = Vec::<TokenStream>::new();

  for item in &exports_trait.items {
    let TraitFn {
      ident,
      inputs,
      inputs_without_types,
      output,
      mangled_name,
    } = for_each_trait_item(trait_name, item);

    let return_type = output_to_return_type!(output);

    let panic_message =
      format!(r#"Failed to get "{ident}" fn symbol from module (mangled name: "{mangled_name}")"#);

    export_inits.push(quote! {
      #ident: unsafe {
        *library.get(concat!(#mangled_name, "\0").as_bytes()).expect(#panic_message)
      }
    });

    let (decl, impl_) = if pub_exports {
      (
        quote! {
          #ident: unsafe extern "C" fn(
            ____return_value____: *mut std::mem::MaybeUninit<#return_type>,
            #inputs
          ) -> bool
        },
        quote! {
          /// Returns `None` if module panics.
          /// Consider unloading module if it panicked, as it is unsafe to call it again.
          /// Note: not all panics are handled, see a ["double panic"](https://doc.rust-lang.org/std/ops/trait.Drop.html#panics).
          /// ```
          /// struct Bomb;
          /// impl Drop for Bomb {
          ///   fn drop(&mut self) {
          ///     panic!("boom"); // will abort the program
          ///   }
          /// }
          /// let _bomb = Bomb;
          /// panic!();
          /// ```
          #[doc = #SAFETY_DOC]
          pub unsafe fn #ident<'module>( &'module self, #inputs ) -> Option<ModuleValue<'module, #return_type>> {
            use std::mem::MaybeUninit;

            let mut ____return_value____ = MaybeUninit::<#return_type>::uninit();

            let success = (self.#ident)(
              &mut ____return_value____,
              #inputs_without_types
            );
            if !success {
              return None;
            }

            // SAFETY: function returned true so we are allowed to read the pointer
            let return_value = unsafe {
              ____return_value____.assume_init_read()
            };
            Some(ModuleValue::new(return_value))
          }
        },
      )
    } else {
      (
        quote! {
          #ident: unsafe extern "C" fn( #inputs ) #output
        },
        quote! {
          #[doc = #SAFETY_DOC]
          pub unsafe fn #ident<'module>( &'module self, #inputs ) -> ModuleValue<'module, #return_type> {
            let return_value = (self.#ident)( #inputs_without_types );
            ModuleValue::new(return_value)
          }
        },
      )
    };

    export_decls.push(decl);
    export_impls.push(impl_);
  }

  let types_import_crate = if pub_exports {
    quote! { relib_host }
  } else {
    quote! { crate }
  };

  write_code_to_file(
    "generated_module_exports.rs",
    quote! {
      #module_use_items

      use #types_import_crate::exports_types::ModuleExportsForHost;
      #[allow(unused_imports)]
      use #types_import_crate::exports_types::ModuleValue;

      pub struct ModuleExports {
        #( #export_decls, )*
      }

      impl ModuleExports {
        pub fn new(library: &libloading::Library) -> Self {
          Self {
            #( #export_inits, )*
          }
        }

        #( #export_impls )*
      }

      impl ModuleExportsForHost for ModuleExports {
        fn new(library: &libloading::Library) -> Self {
          Self::new(library)
        }
      }
    },
  );
}

fn generate_imports(imports_file_content: &'static str, imports_trait_path: &str) {
  let trait_name = extract_trait_name_from_path(imports_trait_path);
  let (imports_trait, module_use_items) =
    parse_trait_file(trait_name, imports_file_content, imports_trait_path);

  let imports_trait_path: syn::Path =
    syn::parse_str(imports_trait_path).expect("Failed to parse imports_trait_path as syn::Path");

  let mut imports = Vec::<TokenStream>::new();

  for item in imports_trait.items {
    let TraitFn {
      ident,
      inputs,
      inputs_without_types,
      output,
      mangled_name,
    } = for_each_trait_item(trait_name, &item);

    let panic_message =
      format!(r#"Failed to get "{mangled_name}" symbol of static function pointer from module"#);

    imports.push(quote! {
      unsafe {
        let ptr: *mut unsafe extern "C" fn( #inputs ) #output
          = *library.get(concat!(#mangled_name, "\0").as_bytes()).expect(#panic_message);

        *ptr = impl_;

        unsafe extern "C" fn impl_( #inputs ) #output {
          <ModuleImportsImpl as Imports>::#ident( #inputs_without_types )
        }
      }
    });
  }

  write_code_to_file(
    "generated_module_imports.rs",
    quote! {
      #module_use_items

      use #imports_trait_path as Imports;

      /// Struct for implementing your `Imports` trait
      pub struct ModuleImportsImpl;

      pub fn init_imports(library: &libloading::Library) {
        #( #imports )*
      }
    },
  );
}
