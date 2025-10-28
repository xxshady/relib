use {
  proc_macro::TokenStream,
  quote::{ToTokens, quote},
  syn::{ItemFn, ReturnType, spanned::Spanned},
};

#[proc_macro_attribute]
pub fn main_export(_args: TokenStream, input: TokenStream) -> TokenStream {
  let fn_item = {
    let input = input.clone();
    syn::parse_macro_input!(input as ItemFn)
  };
  let fn_sig = fn_item.sig;

  if !fn_sig.inputs.is_empty() {
    return compile_error(fn_sig.inputs, "main function can't have any arguments");
  }

  let return_type = fn_output_to_return_type_string(&fn_sig.output);
  if return_type != "()" {
    return compile_error(fn_sig.output, "main function can't return anything");
  }

  let exportified_fn = relib_exportify::exportify(input.into());

  quote! {
    // NOTE ⚠️: relib_module must be imported because it exports internal symbols that required for relib_host crate
    use relib_module as _;

    #exportified_fn
  }
  .into()
}

fn fn_output_to_return_type_string(output: &ReturnType) -> String {
  match output {
    ReturnType::Default => quote! { () }.to_string(),
    ReturnType::Type(_, ty) => ty.to_token_stream().to_string(),
  }
}

fn compile_error(spanned: impl Spanned, message: &str) -> TokenStream {
  syn::Error::new(spanned.span(), message)
    .to_compile_error()
    .into()
}
