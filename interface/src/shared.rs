use std::{fs, path::Path};

use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{format_ident, quote, ToTokens};
use syn::{
  punctuated::Punctuated, FnArg, Ident, Item, ItemTrait, ReturnType, Token, TraitItem, UseTree,
  ItemUse,
};

use relib_internal_shared::fn_inputs_without_types;

pub fn format_code(code: &str, file_name: &str) -> String {
  match syn::parse_file(code) {
    Ok(file) => prettyplease::unparse(&file),
    Err(e) => {
      eprintln!("Failed to parse Rust code for formatting of file: {file_name}, reason: {e:#}");
      code.to_owned()
    }
  }
}

pub fn parse_trait_file(
  trait_name: &str,
  file_content: &'static str,
  trait_path: &str,
) -> (ItemTrait, TokenStream2) {
  let Some((crate_name, _)) = trait_path.split_once("::") else {
    panic!("Failed to extract crate name from trait path: {trait_path}");
  };
  let crate_name = format_ident!("{crate_name}");

  let ast = syn::parse_file(file_content).unwrap_or_else(|e| {
    panic!("Failed to parse Rust code of trait file: {trait_name}: {e:#?}");
  });

  let items = ast.items;

  let module_use_items = items
    .iter()
    .filter_map(|item| {
      let Item::Use(item_use) = item else {
        return None;
      };

      Some(patch_item_use_if_needed(item_use, &crate_name))
    })
    .collect::<TokenStream2>();

  let are_there_any_other_items = items
    .iter()
    .any(|item| !matches!(item, Item::Use(..) | Item::Trait(..)));
  if are_there_any_other_items {
    panic!("Only `use` and `trait` items allowed in the {trait_name} trait file");
  }

  let trait_ = items.into_iter().find_map(|item| {
    let Item::Trait(trait_) = item else {
      return None;
    };
    Some(trait_)
  });

  let Some(trait_) = trait_ else {
    panic!("Expected trait item");
  };

  assert_eq!(
    trait_.ident, trait_name,
    r#"Trait must be named "{trait_name}""#
  );

  (trait_.clone(), module_use_items)
}

pub fn write_code_to_file(file: &str, code: TokenStream2) {
  let code = code.to_string();
  let code = format_code(&code, file);
  let out = format!(
    "// This file is generated, DO NOT edit manually\n\
    // ---------------------------------------------\n\n\
    {code}"
  );

  let out_dir = std::env::var("OUT_DIR").unwrap();
  let path = Path::new(&out_dir).join(file);

  fs::write(&path, out).unwrap_or_else(|e| {
    panic!("Failed to write {}, reason: {e:#}", path.display());
  });
}

pub struct TraitFn<'a> {
  pub ident: &'a Ident,
  pub inputs: &'a Punctuated<FnArg, Token![,]>,
  pub inputs_without_types: Vec<TokenStream2>,
  pub output: &'a ReturnType,
  pub mangled_name: String,
  pub mangled_ident: Ident,

  pub post_ident: Ident,
  pub post_mangled_name: String,
  pub post_mangled_ident: Ident,
}

pub fn for_each_trait_item<'trait_>(
  trait_name: &str,
  trait_item: &'trait_ TraitItem,
) -> TraitFn<'trait_> {
  let TraitItem::Fn(fn_) = trait_item else {
    panic!("All trait items must be functions");
  };
  let fn_ = &fn_.sig;
  assert!(
    fn_.receiver().is_none(),
    "Functions in {trait_name} trait must not have `self` receiver"
  );

  assert!(
    fn_.generics.lt_token.is_none(),
    "Functions in {trait_name} trait must not have generics since they are not FFI-safe"
  );

  let ident = &fn_.ident;

  let inputs_without_types = fn_inputs_without_types!(fn_.inputs);

  // !!! keep in sync with main and before_unload calls in relib_host crate !!!
  let mangled_name = format!("__relib__{trait_name}_{ident}");
  let mangled_ident = format_ident!("{mangled_name}");
  let post_mangled_name = format!("__post{mangled_name}");
  let post_mangled_ident = format_ident!("{post_mangled_name}");

  TraitFn {
    ident,
    inputs: &fn_.inputs,
    inputs_without_types,
    output: &fn_.output,
    mangled_name,
    mangled_ident,
    post_ident: format_ident!("post_{ident}"),
    post_mangled_name,
    post_mangled_ident,
  }
}

pub fn extract_trait_name_from_path(trait_path: &str) -> &str {
  trait_path.split("::").last().unwrap_or_else(|| {
    panic!("Failed to extract trait name from path: {trait_path}");
  })
}

fn patch_item_use_if_needed(item_use: &ItemUse, crate_name: &Ident) -> TokenStream2 {
  match &item_use.tree {
    UseTree::Path(path) => {
      let ident = path.ident.to_string();
      let ident = ident.as_str();

      match ident {
        "super" => {
          let code = item_use.to_token_stream();
          panic!(
            "Failed to copy `{code}`\n\
            note: `use super::` syntax is not supported, use absolute imports, for example `use crate::something`"
          );
        }
        "crate" => {
          let tree = &path.tree;
          quote! {
            use #crate_name::#tree;
          }
        }
        _ => item_use.to_token_stream(),
      }
    }
    _ => {
      let code = item_use.to_token_stream();
      panic!("unexpected syntax: `{code}`");
    }
  }
}

pub const SAFETY_DOC: &str = "# Safety\n\
  Behavior is undefined if any of the following conditions are violated:\n\
  1. Types of arguments and return value must be FFI-safe.\n\
  2. Host and module crates must be compiled with same shared crate code (which contains exports and imports traits).\n\
  3. Returned value must not be a reference-counting pointer (see [limitations](https://docs.rs/relib/latest/relib/#moving-non-copy-types-between-host-and-module)).";

pub fn type_needs_box(type_: &TokenStream2) -> bool {
  relib_internal_shared::type_needs_box(&type_.to_string())
}
