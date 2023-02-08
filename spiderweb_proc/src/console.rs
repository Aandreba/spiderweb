use proc_macro2::{TokenStream};
use syn::{parse::Parse, punctuated::Punctuated, Expr, Token, LitStr};

pub enum Console {
    Format(TokenStream),
    Value(Punctuated<Expr, Token![,]>),
}

impl Parse for Console {
    #[inline]
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.fork().parse::<LitStr>().is_ok() {
            return input.parse::<TokenStream>().map(Self::Format)
        } else {
            return Punctuated::parse_terminated(input).map(Self::Value)
        }
    }
}
