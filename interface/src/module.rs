use {
  crate::shared::{
    SAFETY_DOC, TRANSFER_IMPORTS_TO_HOST, TRANSFER_IMPORTS_TO_HOST_INTERNAL, TraitFn,
    extract_trait_name_from_path, for_each_trait_item, out_dir_file_name, parse_trait_file,
    pass_out_dir_file_name_to_crate_code, type_needs_box, write_code_to_file,
  },
  proc_macro2::TokenStream as TokenStream2,
  quote::quote,
  relib_internal_shared::output_to_return_type,
  syn::FnArg,
};

#[cfg(feature = "internal")]
pub fn generate_internal(
  exports_file_content: &'static str,
  exports_trait_path: &str,
  imports_file_content: &'static str,
  imports_trait_path: &str,
) {
  generate_exports_(
    exports_file_content,
    exports_trait_path,
    false,
    "internal_generated_module",
  );
  generate_imports_(
    imports_file_content,
    imports_trait_path,
    false,
    "internal_generated_module",
  );
}

/// in the OUT_DIR which you can include using
/// `relib_interface::include_exports!();` and `relib_interface::include_exports!();`
/// in your `lib.rs` or `main.rs`
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
  generate_exports_(
    exports_file_content,
    exports_trait_path,
    true,
    "generated_module",
  );
  generate_imports_(
    imports_file_content,
    imports_trait_path,
    true,
    "generated_module",
  );
}

/// in the OUT_DIR which you can include using
/// `relib_interface::include_exports!(gen_exports, <prefix>);` and `relib_interface::include_imports!(gen_imports, <prefix>);`
/// in your `lib.rs` or `main.rs`
/// and then use `ModuleExportsImpl` struct to implement your `Exports` trait:
/// ```
/// impl Exports for ModuleExportsImpl {
///   // ...
/// }
/// ```
#[cfg(feature = "public")]
pub fn generate_with_prefix(
  prefix: &str,
  exports_file_content: &'static str,
  exports_trait_path: &str,
  imports_file_content: &'static str,
  imports_trait_path: &str,
) {
  generate_exports_(exports_file_content, exports_trait_path, true, prefix);
  generate_imports_(imports_file_content, imports_trait_path, true, prefix);
}

#[cfg(feature = "public")]
pub fn generate_exports(exports_file_content: &'static str, exports_trait_path: &str) {
  generate_exports_(
    exports_file_content,
    exports_trait_path,
    true,
    "generated_module",
  );
}

#[cfg(feature = "public")]
pub fn generate_imports(imports_file_content: &'static str, imports_trait_path: &str) {
  generate_imports_(
    imports_file_content,
    imports_trait_path,
    true,
    "generated_module",
  );
}

#[cfg(feature = "public")]
pub fn generate_exports_with_prefix(
  prefix: &str,
  exports_file_content: &'static str,
  exports_trait_path: &str,
) {
  generate_exports_(exports_file_content, exports_trait_path, true, prefix);
}

#[cfg(feature = "public")]
pub fn generate_imports_with_prefix(
  prefix: &str,
  imports_file_content: &'static str,
  imports_trait_path: &str,
) {
  generate_imports_(imports_file_content, imports_trait_path, true, prefix);
}

fn generate_exports_(
  exports_file_content: &'static str,
  exports_trait_path: &str,
  pub_exports: bool,
  prefix: &str,
) {
  pass_out_dir_file_name_to_crate_code(prefix, "exports");

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
      lifetimes_where_module: _,
      lifetimes_module: _,
    } = for_each_trait_item(trait_name, &item);

    let transfer_imports = TRANSFER_IMPORTS_TO_HOST.clone();

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
            #[allow(clippy::extra_unused_lifetimes)]
            pub fn #post_mangled_ident #lifetimes_full (
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
            #transfer_imports
            unsafe { Transfer::<TransferToHost>::transfer(&return_value, &()) }
            return_value
          },
          quote! {},
        )
      };

      quote! {
        #[unsafe(no_mangle)]
        #[allow(non_snake_case)]
        pub fn #mangled_ident #lifetimes_full (
          ____success____: *mut bool,
          #inputs
        ) -> std::mem::MaybeUninit<#return_type> // will be initialized if function won't panic
        {
          let result = std::panic::catch_unwind(move || {
            <ModuleExportsImpl as Exports>::#ident( #( #inputs_without_types, )* )
          });

          match result {
            Ok(return_value) => {
              unsafe {
                *____success____ = true;
              }

              #[allow(unused_braces, clippy::unit_arg)]
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
        pub fn #ident #lifetimes_full ( #inputs ) #output {
          <ModuleExportsImpl as Exports>::#ident( #( #inputs_without_types, )* )
        }
      }
    };

    exports.push(code);
  }

  write_code_to_file(
    &out_dir_file_name(prefix, "exports"),
    quote! {
      #module_use_items

      use #exports_trait_path as Exports;

      /// Struct for implementing your `Exports` trait
      pub struct ModuleExportsImpl;

      #( #exports )*
    },
  );
}

fn generate_imports_(
  imports_file_content: &'static str,
  imports_trait_path: &str,
  pub_imports: bool,
  prefix: &str,
) {
  pass_out_dir_file_name_to_crate_code(prefix, "imports");

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
      lifetimes_where_module: _,
      lifetimes_module: _,
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

    let transfer_imports_pub = TRANSFER_IMPORTS_TO_HOST.clone();
    let transfer_imports_internal = TRANSFER_IMPORTS_TO_HOST_INTERNAL.clone();

    let transfer_inputs = quote! {
      #(
        unsafe {
          Transfer::<TransferToHost>::transfer(
            &#inputs_without_types,
            &()
          );
        };
      )*
    };
    let transfer_inputs_pub = quote! {
      {
        #transfer_imports_pub
        #transfer_inputs
      }
    };
    let transfer_inputs_internal = quote! {
      {
        #transfer_imports_internal
        #transfer_inputs
      }
    };

    let transfer_return_value_module_id = quote! {
      ____transfer_module_id_____: relib_shared::ModuleId
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
            static mut #post_mangled_ident: #lifetimes_for fn(return_value_ptr: *mut #return_type) = ____post_placeholder____;

            #[allow(clippy::extra_unused_lifetimes)]
            fn ____post_placeholder____ #lifetimes_full (_: *mut #return_type) {
              unreachable!();
            }
          },
          quote! {
            // SAFETY: function returned true so we are allowed to read the pointer
            let return_ptr = unsafe { return_value.assume_init() };
            let return_value: #return_type = unsafe {
              Clone::clone(&*return_ptr)
            };

            // SAFETY: user of relib_interface crate should guarantee that
            // module and host binaries are compiled with the same shared crate code
            unsafe {
              #post_mangled_ident(return_ptr);
            }

            return_value
          },
        )
      } else {
        (
          return_type,
          quote! {},
          quote! {
            unsafe {
              return_value.assume_init()
            }
          },
        )
      };

      quote! {
        #function_static_decl: #lifetimes_for fn(
          ____success____: *mut bool,
          #transfer_return_value_module_id,
          #inputs
        ) -> std::mem::MaybeUninit<#return_type> = ____placeholder____;

        fn ____placeholder____ #lifetimes_full (
          ____success____: *mut bool,
          #transfer_return_value_module_id,
          #placeholder_inputs
        ) -> std::mem::MaybeUninit<#return_type> {
          unreachable!();
        }

        #post_fn_decl

        let mut ____success____ = std::mem::MaybeUninit::<bool>::uninit();

        #transfer_inputs_pub

        #suppress_lints_for_return_value
        let return_value = unsafe {
          #mangled_ident(
            ____success____.as_mut_ptr(),
            relib_module::__internal::module_id(),
            #( #inputs_without_types, )*
          )
        };

        // SAFETY: this bool is guaranteed to be initialized by the host
        if !unsafe{ ____success____.assume_init() } {
          // TODO: expose unrecoverable helper in relib_module::__internal and use it here?
          eprintln!("[relib] host panicked while executing import {:?} of module, aborting", stringify!(#ident));
          std::process::abort();
        }

        // SAFETY: function returned true so we are allowed to read the return value
        #read_return_value
      }
    } else {
      quote! {
        #function_static_decl: #lifetimes_for fn(
          #transfer_return_value_module_id,
          #inputs
        ) #output = placeholder;

        fn placeholder #lifetimes_full (
          #transfer_return_value_module_id,
          #placeholder_inputs
        ) #output {
          unreachable!();
        }

        #transfer_inputs_internal

        unsafe {
          #mangled_ident(
            crate::__internal::module_id(),
            #( #inputs_without_types, )*
          )
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
    &out_dir_file_name(prefix, "imports"),
    quote! {
      #module_use_items

      #( #imports )*
    },
  );
}
