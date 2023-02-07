use derive_syn_parse::Parse;
use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use syn::{braced, ext::IdentExt, parse::Parse, spanned::Spanned, Expr, Path, Token};

pub enum Content {
    Element(Element),
    Expr(Expr),
}

pub struct Element {
    pub open: OpenTag,
    pub content: Vec<Content>,
    pub close: Option<CloseTag>,
}

#[derive(Parse)]
pub struct Attribute {
    pub name: Ident,
    pub eq_token: Token![=],
    #[brace]
    pub brace_token: syn::token::Brace,
    #[inside(brace_token)]
    pub value: Expr,
}

#[derive(Parse)]
pub struct OpenTag {
    pub open_bracket: Token![<],
    pub path: Path,
    #[call(parse_attributes)]
    pub attrs: Vec<Attribute>,
    pub end_bracket: Option<Token![/]>,
    pub close_bracket: Token![>],
}

#[derive(Parse)]
pub struct CloseTag {
    pub open_bracket: Token![<],
    pub end_bracker: Token![/],
    pub path: Path,
    pub close_bracket: Token![>],
}

impl Content {
    #[inline]
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Option<Self>> {
        if input.peek(Token![<]) {
            if input.peek2(Token![/]) {
                return Ok(None);
            } else {
                return Ok(Some(Self::Element(input.parse::<Element>()?)));
            }
        } else if input.peek(syn::token::Brace) {
            let content;
            braced!(content in input);
            return Ok(Some(Self::Expr(content.parse::<Expr>()?)));
        }

        return Err(input.error("Unknown element"));
    }

    #[inline]
    fn render(&self) -> TokenStream {
        match self {
            Self::Element(x) => x.to_token_stream(),
            Self::Expr(x) => x.to_token_stream(),
        }
    }
}

impl Parse for Element {
    #[inline]
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let open = OpenTag::parse(input)?;
        if open.end_bracket.is_some() {
            return Ok(Self {
                open,
                content: Vec::new(),
                close: None,
            });
        }

        let mut content = Vec::new();
        while let Some(x) = Content::parse(input)? {
            content.push(x)
        }

        let close = match input.peek(Token![<]) {
            true => {
                let close = CloseTag::parse(input)?;
                if close.path != open.path {
                    return Err(syn::Error::new(
                        close
                            .path
                            .span()
                            .join(open.path.span())
                            .unwrap_or_else(|| close.path.span()),
                        "Paths don't match",
                    ));
                }
                close
            }

            false => {
                return Err(syn::Error::new(
                    input
                        .span()
                        .join(open.path.span())
                        .unwrap_or_else(|| input.span()),
                    "Element not closed",
                ))
            }
        };

        return Ok(Self {
            open,
            content,
            close: Some(close),
        });
    }
}

impl ToTokens for Element {
    #[inline]
    fn to_tokens(&self, tokens: &mut TokenStream) {
        return match self.open.path.get_ident() {
            Some(x) if x.to_string().starts_with(char::is_lowercase) => {
                client_primitive(self, tokens)
            }
            _ => client_component(self, tokens),
        }
        .into();
    }
}

fn client_primitive(Element { open, content, .. }: &Element, tokens: &mut TokenStream) {
    let path = &open.path;

    if content.is_empty() {
        return tokens.extend(quote! {
            ::std::result::Result::<::spiderweb::dom::Element::<_>, ::spiderweb::wasm_bindgen::JsValue>::Ok(::spiderweb::dom::Element::new(stringify!(#path), ()))
        });
    }

    let content = content.iter().map(Content::render);
    return tokens.extend(quote! {
        (|| {
            ::std::result::Result::<::spiderweb::dom::Element::<_>, ::spiderweb::wasm_bindgen::JsValue>::Ok(
                ::spiderweb::dom::Element::new(stringify!(#path), ()).
                    #(append_child_inner(#content)?).*
            )
        })()
    });
}

fn client_component(Element { open, content, .. }: &Element, tokens: &mut TokenStream) {
    let path = &open.path;
    let attrs = open.attrs.iter().map(|Attribute { name, value, .. }| {
        quote! {
           #name: #value
        }
    });

    if content.is_empty() {
        return tokens.extend(quote! {
            ::spiderweb::dom::Component::render(#path { #(#attrs),* })
        });
    }

    let content = content.iter().map(Content::render);
    tokens.extend(quote! {
        (|| {
            ::std::result::Result::<::spiderweb::dom::Element::<_>, ::spiderweb::wasm_bindgen::JsValue>::Ok(
                ::spiderweb::dom::Component::render(#path { #(#attrs),* })?.
                #(append_child_inner(#content)?).*
            )
        })()
    });
}

#[inline]
fn parse_attributes(input: syn::parse::ParseStream) -> syn::Result<Vec<Attribute>> {
    let mut result = Vec::new();
    while input.peek(Ident::peek_any) {
        result.push(input.parse()?)
    }
    return Ok(result);
}
