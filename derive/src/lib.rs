extern crate proc_macro;

use {
  proc_macro::TokenStream,
  quote::quote,
  syn::{Data, DeriveInput, Fields, parse_macro_input},
};

#[proc_macro_derive(Transfer)]
pub fn transfer_derive(input: TokenStream) -> TokenStream {
  let input = parse_macro_input!(input as DeriveInput);
  let name = &input.ident;
  let mut generics = input.generics.clone();
  generics
    .params
    .insert(0, syn::parse_quote!(F: ::relib_shared::TransferTarget));
  let (impl_generics, _, _) = generics.split_for_impl();
  let (_, ty_generics, where_clause) = input.generics.split_for_impl();

  let transfer = quote! {
    ::relib_shared::Transfer::<F>::transfer
  };

  let transfer_calls = match &input.data {
    Data::Struct(data) => match &data.fields {
      Fields::Named(fields) => {
        let calls = fields.named.iter().map(|f| {
          let name = &f.ident;
          quote! {
            unsafe { #transfer(&self.#name, ctx); }
          }
        });
        quote! {
          #(#calls)*
        }
      }
      Fields::Unnamed(fields) => {
        let field_indices = fields
          .unnamed
          .iter()
          .enumerate()
          .map(|(i, _)| syn::Index::from(i));
        quote! {
          #(
            unsafe { #transfer(&self.#field_indices, ctx); }
          )*
        }
      }
      Fields::Unit => quote!(),
    },
    Data::Enum(data) => {
      let variant_transfers = data.variants.iter().map(|v| {
        let variant_ident = &v.ident;
        match &v.fields {
          Fields::Named(fields) => {
            let field_names = fields.named.iter().map(|f| &f.ident);
            let field_names_clone = field_names.clone();
            let calls = field_names.map(|name| {
              quote! { unsafe { #transfer(#name, ctx) }; }
            });
            quote! {
              Self::#variant_ident { #(#field_names_clone),* } => {
                #(#calls)*
              }
            }
          }
          Fields::Unnamed(fields) => {
            let field_names = (0..fields.unnamed.len())
              .map(|i| syn::Ident::new(&format!("field{}", i), v.ident.span()));
            let field_names_clone = field_names.clone();
            let calls = field_names.map(|name| {
              quote! { unsafe { #transfer(#name, ctx) }; }
            });
            quote! {
              Self::#variant_ident ( #(#field_names_clone),* ) => {
                #(#calls)*
              }
            }
          }
          Fields::Unit => {
            quote! {
              Self::#variant_ident => {}
            }
          }
        }
      });
      quote! {
        match self {
          #(#variant_transfers)*
        }
      }
    }
    Data::Union(_) => unimplemented!("Transfer derive macro does not support unions."),
  };

  let expanded = quote! {
    unsafe impl #impl_generics ::relib_shared::Transfer<F> for #name #ty_generics #where_clause {
      unsafe fn transfer(&self, ctx: &F::ExtraContext) {
        #transfer_calls
      }
    }
  };

  expanded.into()
}
