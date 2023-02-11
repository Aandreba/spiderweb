use derive_syn_parse::Parse;
use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use syn::{
    braced, custom_keyword, ext::IdentExt, parse::Parse, spanned::Spanned, Expr, Path, Token,
};
custom_keyword!(on);

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
    #[call(parse_on)]
    pub on_token: Option<(on, Token![:])>,
    pub name: Ident,
    #[peek(Token![=])]
    pub value: Option<AttributeValue>,
}

#[derive(Parse)]
pub struct AttributeValue {
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
    fn parse_option(input: syn::parse::ParseStream) -> syn::Result<Option<Self>> {
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
        while let Some(x) = Content::parse_option(input)? {
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
    let mut my_tokens = quote! { ::spiderweb::dom::Element::stateless(stringify!(#path)) };

    // Content
    for content in content.iter() {
        let content = content.render();
        my_tokens.extend(quote! { .append_child_inner(#content)? });
    }

    // Attributes
    for Attribute {
        on_token,
        name,
        value,
    } in open.attrs.iter()
    {
        let value = value
            .as_ref()
            .map_or_else(|| name.to_token_stream(), |x| x.value.to_token_stream());

        match on_token.is_some() {
            true => {
                my_tokens.extend(quote! { .set_callback_inner(stringify!(#name), #value) })
            },
            false => {
                my_tokens.extend(quote! { .set_attribute_inner(stringify!(#name), #value, ::spiderweb::std::string::ToString::to_string)? });
            }
        }
    }

    tokens.extend(quote! {
        (|| ::spiderweb::std::result::Result::<_, ::spiderweb::wasm_bindgen::JsValue>::Ok(#my_tokens))()
    })
}

fn client_component(Element { open, content, .. }: &Element, tokens: &mut TokenStream) {
    let path = &open.path;
    let attrs = open.attrs.iter().map(|Attribute { name, value, .. }| {
        let value = value
            .as_ref()
            .map(|AttributeValue { value, .. }| quote! { :#value });
        quote! {
           #name #value
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

#[inline]
fn parse_on(input: syn::parse::ParseStream) -> syn::Result<Option<(on, Token![:])>> {
    if input.peek(on) {
        let on = input.parse()?;
        let token = input.parse()?;
        return Ok(Some((on, token)));
    }
    return Ok(None);
}
