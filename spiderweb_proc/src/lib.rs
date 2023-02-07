use parse::Element;
use quote::ToTokens;
use syn::parse_macro_input;
mod parse;

#[proc_macro]
pub fn client (items: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let element = parse_macro_input!(items as Element);
    return element.to_token_stream().into()
}