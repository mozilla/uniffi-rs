use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    bracketed,
    parse::{Parse, ParseStream},
    token, LitStr, Token,
};

#[derive(Clone)]
pub enum SwiftProtocols {
    Single(LitStr),
    Multiple(Vec<LitStr>),
}

impl ToTokens for SwiftProtocols {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            SwiftProtocols::Single(lit) => lit.to_tokens(tokens),
            SwiftProtocols::Multiple(items) => tokens.extend(quote! { #(#items),* }),
        }
    }
}

impl Parse for SwiftProtocols {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(token::Bracket) {
            let content;
            let _ = bracketed!(content in input);
            Ok(Self::Multiple(
                content
                    .parse_terminated(|stream| stream.parse::<LitStr>(), Token![,])?
                    .into_iter()
                    .collect(),
            ))
        } else {
            Ok(Self::Single(input.parse()?))
        }
    }
}

impl SwiftProtocols {
    pub(crate) fn to_vec(&self) -> Vec<String> {
        match self {
            SwiftProtocols::Single(lit) => vec![lit.value()],
            SwiftProtocols::Multiple(items) => items.iter().map(|item| item.value()).collect(),
        }
    }
}
