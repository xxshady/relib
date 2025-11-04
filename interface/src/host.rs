use {
  crate::shared::{
    SAFETY_DOC, TRANSFER_IMPORTS_TO_MODULE, TRANSFER_IMPORTS_TO_MODULE_INTERNAL, TraitFn,
    extract_trait_name_from_path, for_each_trait_item, out_dir_file_name, parse_trait_file,
    pass_out_dir_file_name_to_crate_code, type_needs_box, write_code_to_file,
  },
  proc_macro2::TokenStream as TokenStream2,
  quote::quote,
  relib_internal_shared::output_to_return_type,
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

#[cfg(feature = "internal")]
pub fn generate_internal_exports_with_prefix(
  prefix: &str,
  exports_file_content: &'static str,
  exports_trait_path: &str,
) {
  generate_exports_(exports_file_content, exports_trait_path, false, prefix);
}

/// Will generate `generated_module_exports.rs` and `generated_module_imports.rs`
/// in the OUT_DIR which you can include using
/// `relib_interface::include_exports!();` and `relib_interface::include_exports!();`
/// in your `lib.rs` or `main.rs` and then use `ModuleExports` struct:
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

/// Will generate `{prefix}_exports.rs` and `{prefix}_imports.rs` in the OUT_DIR which you can include
/// using `relib_interface::include_exports!(gen_exports, <prefix>);` and `relib_interface::include_imports!(gen_imports, <prefix>);`
/// in your `lib.rs` or `main.rs` and then use `ModuleExports` struct:
/// ```
/// let exports = ModuleExports::new(library);
/// exports.something();
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

  let mut export_decls = Vec::<TokenStream2>::new();
  let mut export_inits = Vec::<TokenStream2>::new();
  let mut export_impls = Vec::<TokenStream2>::new();

  for item in &exports_trait.items {
    let TraitFn {
      ident,
      inputs,
      inputs_without_types,
      output,
      mangled_name,
      mangled_ident: _,
      post_ident,
      post_mangled_name,
      post_mangled_ident: _,
      lifetimes_for,
      lifetimes_full: _,
      lifetimes_where_module,
      lifetimes_module,
    } = for_each_trait_item(trait_name, item);

    let pub_return_type = output_to_return_type!(output);

    let panic_message = format!(
      "Couldn't find \"{ident}\" export\n\
      note: make sure this module implements {exports_trait_path} trait\n\
      note: if `relib_interface::include_exports!()` is used not in module crate, \
      make sure the crate (that invokes `include_exports`) is used in module crate, for example you can do so by \
      using this syntax: `use <crate> as _;`"
    );

    let post_panic_message = format!(
      r#"Failed to get "{post_ident}" fn symbol from module (mangled name: "{post_mangled_name}")"#
    );

    let import_init = quote! {
      #ident: unsafe {
        *library.get(concat!(#mangled_name, "\0").as_bytes()).expect(#panic_message)
      },
    };

    let ignore_code_style_warns = quote! {
      #[allow(clippy::needless_lifetimes)]
    };

    let transfer_imports_pub = TRANSFER_IMPORTS_TO_MODULE.clone();
    let transfer_imports_internal = TRANSFER_IMPORTS_TO_MODULE_INTERNAL.clone();

    let transfer_inputs = quote! {
      #(
        unsafe {
          Transfer::<TransferToModule>::transfer(
            &#inputs_without_types,
            self.____module_id____,
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

    // !!! keep in sync with main and before_unload calls in relib_host crate !!!
    let (decl, init, impl_) = if pub_exports {
      let needs_box = type_needs_box(&pub_return_type);
      let (post_decl, post_init, return_type, read_return_value) = if needs_box {
        (
          quote! {
            #post_ident: #lifetimes_for fn( *mut #pub_return_type ),
          },
          quote! {
            #post_ident: unsafe {
              *library.get(concat!(#post_mangled_name, "\0").as_bytes()).expect(#post_panic_message)
            },
          },
          quote! { *mut #pub_return_type },
          quote! {
            let return_ptr = unsafe { return_value.assume_init() };

            let return_value: #pub_return_type = unsafe {
              Clone::clone(&*return_ptr)
            };
            (self.#post_ident)(return_ptr);

            return_value
          },
        )
      } else {
        (
          quote! {},
          quote! {},
          pub_return_type.clone(),
          quote! { unsafe { return_value.assume_init() } },
        )
      };

      (
        quote! {
          #ident: #lifetimes_for fn(
            ____success____: *mut bool,
            #inputs
          ) -> std::mem::MaybeUninit<#return_type>,
          #post_decl
        },
        quote! {
          #import_init
          #post_init
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
          #[must_use = "returns None if module panics, consider unloading module if it panicked, as it is unsafe to call it again"]
          #ignore_code_style_warns
          pub unsafe fn #ident <'module, #lifetimes_module> (
            &'module self,
            #inputs
          ) -> Option<#pub_return_type>
          #lifetimes_where_module
          {
            #transfer_inputs_pub

            let mut ____success____ = std::mem::MaybeUninit::<bool>::uninit();

            let return_value = (self.#ident)(
              ____success____.as_mut_ptr(),
              #( #inputs_without_types, )*
            );

            // SAFETY: this bool is guaranteed to be initialized by the module
            if !unsafe { ____success____.assume_init() } {
              return None;
            }

            // SAFETY: function returned true so we are allowed to read the pointer
            #[allow(unused_braces, clippy::unit_arg)]
            Some({ #read_return_value })
          }
        },
      )
    } else {
      (
        quote! {
          pub(crate) #ident: #lifetimes_for fn( #inputs ) #output,
        },
        import_init,
        quote! {
          #[doc = #SAFETY_DOC]
          #ignore_code_style_warns
          pub unsafe fn #ident #lifetimes_module ( &self, #inputs ) -> #pub_return_type
          #lifetimes_where_module
          {
            #transfer_inputs_internal

            #[allow(clippy::let_unit_value)]
            let return_value = (self.#ident)( #( #inputs_without_types, )* );
            return_value
          }
        },
      )
    };

    export_decls.push(decl);
    export_inits.push(init);
    export_impls.push(impl_);
  }

  let types_import_crate = if pub_exports {
    quote! { relib_host }
  } else {
    quote! { crate }
  };

  write_code_to_file(
    &out_dir_file_name(prefix, "exports"),
    quote! {
      #module_use_items

      use #types_import_crate::{
        exports_types::ModuleExportsForHost,
        // renaming it to avoid name collision with user code
        ModuleId as RelibModuleId,
      };

      #[allow(non_snake_case)]
      pub struct ModuleExports {
        #[allow(unused)]
        ____module_id____: RelibModuleId,

        #( #export_decls )*
      }

      impl ModuleExports {
        #( #export_impls )*
      }

      impl ModuleExportsForHost for ModuleExports {
        fn new(library: &libloading::Library, ____module_id____: RelibModuleId) -> Self {
          Self {
            ____module_id____,
            #( #export_inits )*
          }
        }
      }
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

  let imports_trait_path_syn: syn::Path =
    syn::parse_str(imports_trait_path).expect("Failed to parse imports_trait_path as syn::Path");

  let mut imports = Vec::<TokenStream2>::new();

  for item in imports_trait.items {
    let TraitFn {
      ident,
      inputs,
      inputs_without_types,
      output,
      mangled_name,
      mangled_ident: _,
      post_ident: _,
      post_mangled_name,
      post_mangled_ident: _,
      lifetimes_for,
      lifetimes_full,
      lifetimes_where_module: _,
      lifetimes_module: _,
    } = for_each_trait_item(trait_name, &item);

    let panic_message = format!(
      "Couldn't find \"{ident}\" import\n\
      note: make sure this module expects {imports_trait_path} trait\n\
      note: if `relib_interface::include_imports!()` is used not in module crate, \
      make sure (that invokes `include_imports`) is used in module crate, for example you can do so by \
      using this syntax: `use <crate> as _;`"
    );

    let post_panic_message = format!(
      r#"Failed to get "{post_mangled_name}" symbol of static function pointer from module"#
    );

    let transfer_imports_pub = TRANSFER_IMPORTS_TO_MODULE.clone();
    let transfer_imports_internal = TRANSFER_IMPORTS_TO_MODULE_INTERNAL.clone();

    let transfer_return_value = quote! {
      {
        unsafe {
          Transfer::<TransferToModule>::transfer(
            &return_value,
            ____transfer_module_id_____,
          );
        };
        return_value
      }
    };

    let return_type = output_to_return_type!(output);

    let (transfer_return_value_pub, transfer_return_value_internal) =
      if return_type.to_string() != "!" {
        (
          quote! {
            {
              #transfer_imports_pub
              #transfer_return_value
            }
          },
          quote! {
            {
              #transfer_imports_internal
              #transfer_return_value
            }
          },
        )
      } else {
        (quote! {}, quote! {})
      };

    let transfer_return_value_module_id = quote! {
      ____transfer_module_id_____: relib_shared::ModuleId
    };

    // !!! keep in sync with main and before_unload calls in relib_host crate !!!
    let impl_code = if pub_imports {
      let needs_box = type_needs_box(&return_type);
      // TODO: remove _return_value?
      let (return_type, _return_value, post_init) = if needs_box {
        (
          quote! {
            *mut #return_type
          },
          quote! {
            use std::boxed::Box;
            Box::into_raw(Box::new(return_value))
          },
          quote! {
            let post_ptr: *mut #lifetimes_for fn(return_value_ptr: *mut #return_type)
              = *library.get(concat!(#post_mangled_name, "\0").as_bytes()).expect(#post_panic_message);

            *post_ptr = post_impl;

            #[allow(clippy::extra_unused_lifetimes)]
            fn post_impl #lifetimes_full (return_value_ptr: *mut #return_type) {
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
        unsafe {
          let ptr: *mut #lifetimes_for fn(
            ____success____: *mut bool,
            #transfer_return_value_module_id,
            #inputs
          ) -> std::mem::MaybeUninit<#return_type>
            = *library.get(concat!(#mangled_name, "\0").as_bytes()).expect(#panic_message);

          *ptr = impl_;

          fn impl_ #lifetimes_full (
            ____success____: *mut bool,
            #transfer_return_value_module_id,
            #inputs
          ) -> std::mem::MaybeUninit<#return_type> // will be initialized if function won't panic
          {
            let result = std::panic::catch_unwind(move || {
              <ModuleImportsImpl as Imports>::#ident( #( #inputs_without_types, )* )
            });

            match result {
              Ok(return_value) => {
                unsafe {
                  *____success____ = true;
                }

                #[allow(unused_braces, clippy::unit_arg)]
                std::mem::MaybeUninit::new({ #transfer_return_value_pub })
              }
              // ignoring content since it's handled in default panic hook of std
              Err(_) => {
                unsafe {
                  *____success____ = false;
                }

                std::mem::MaybeUninit::uninit()
              }
            }
          }

          #post_init
        }
      }
    } else {
      quote! {
        unsafe {
          let ptr: *mut #lifetimes_for fn(
            #transfer_return_value_module_id,
            #inputs
          ) #output
            = *library.get(concat!(#mangled_name, "\0").as_bytes()).expect(#panic_message);

          *ptr = impl_;

          fn impl_ #lifetimes_full (
            #transfer_return_value_module_id,
            #inputs
          ) #output {
            #[allow(unused_variables)] // for never type
            let return_value = <ModuleImportsImpl as Imports>::#ident( #( #inputs_without_types, )* );
            #transfer_return_value_internal
          }
        }
      }
    };

    imports.push(impl_code);
  }

  write_code_to_file(
    &out_dir_file_name(prefix, "imports"),
    quote! {
      #module_use_items

      use #imports_trait_path_syn as Imports;

      /// Struct for implementing your `Imports` trait
      pub struct ModuleImportsImpl;

      pub fn init_imports(library: &libloading::Library) {
        #( #imports )*
      }
    },
  );
}
