/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::util::{
    either_attribute_arg, ident_to_string, kw, mod_path, parse_comma_separated, UniffiAttributeArgs,
};
use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use syn::{
    braced,
    parse::{Parse, ParseStream},
    spanned::Spanned,
    token::Brace,
    Expr, ExprClosure, Pat, Token, Type,
};

pub struct CustomTypeArgs {
    custom_type: Type,
    uniffi_type: Type,
    options: CustomTypeOptions,
}

impl Parse for CustomTypeArgs {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        // Parse the custom / UniFFI type which are both required
        let custom_type = input.parse()?;
        input.parse::<Token![,]>()?;
        let uniffi_type = input.parse()?;
        let options = if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;

            if input.peek(Brace) {
                let content;
                braced!(content in input);
                content.parse()?
            } else {
                CustomTypeOptions::default()
            }
        } else {
            CustomTypeOptions::default()
        };
        Ok(Self {
            custom_type,
            uniffi_type,
            options,
        })
    }
}

/// Handle extra options for the custom type macro
#[derive(Default)]
pub struct CustomTypeOptions {
    from_custom: Option<ConvertClosure>,
    try_into_custom: Option<ConvertClosure>,
    remote: Option<kw::remote>,
}

impl Parse for CustomTypeOptions {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        parse_comma_separated(input)
    }
}

impl UniffiAttributeArgs for CustomTypeOptions {
    fn parse_one(input: ParseStream<'_>) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::from_custom) {
            input.parse::<kw::from_custom>()?;
            input.parse::<Token![:]>()?;
            Ok(Self {
                from_custom: Some(input.parse()?),
                ..Self::default()
            })
        } else if lookahead.peek(kw::try_into_custom) {
            input.parse::<kw::try_into_custom>()?;
            input.parse::<Token![:]>()?;
            Ok(Self {
                try_into_custom: Some(input.parse()?),
                ..Self::default()
            })
        } else if lookahead.peek(kw::remote) {
            Ok(Self {
                remote: Some(input.parse()?),
                ..Self::default()
            })
        } else {
            Err(lookahead.error())
        }
    }

    fn merge(self, other: Self) -> syn::Result<Self> {
        Ok(Self {
            from_custom: either_attribute_arg(self.from_custom, other.from_custom)?,
            try_into_custom: either_attribute_arg(self.try_into_custom, other.try_into_custom)?,
            remote: either_attribute_arg(self.remote, other.remote)?,
        })
    }
}

struct ConvertClosure {
    closure: ExprClosure,
    param_ident: Ident,
    body: Expr,
}

impl ConvertClosure {
    fn token_tuple(&self) -> (TokenStream, TokenStream) {
        let ConvertClosure {
            param_ident, body, ..
        } = self;
        (quote! { #param_ident }, quote! { #body })
    }
}

impl Parse for ConvertClosure {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let closure: ExprClosure = input.parse()?;
        if closure.inputs.len() != 1 {
            return Err(syn::Error::new(
                closure.inputs.span(),
                "Expected exactly 1 argument",
            ));
        }
        let param_ident = match closure.inputs.last().unwrap().clone() {
            Pat::Ident(i) => Ok(i.ident),
            Pat::Type(p) => match *p.pat {
                Pat::Ident(i) => Ok(i.ident),
                _ => Err(p.pat.span()),
            },
            input => Err(input.span()),
        }
        .map_err(|span| syn::Error::new(span, "Unexpected argument type"))?;
        Ok(Self {
            body: *closure.body.clone(),
            closure,
            param_ident,
        })
    }
}

impl ToTokens for ConvertClosure {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.closure.to_tokens(tokens)
    }
}

pub(crate) fn expand_custom_type(args: CustomTypeArgs) -> syn::Result<TokenStream> {
    let CustomTypeArgs {
        custom_type,
        uniffi_type,
        options,
    } = args;

    let name = match &custom_type {
        Type::Path(p) => match p.path.get_ident() {
            Some(i) => Ok(ident_to_string(i)),
            None => Err("Custom types must only have one component"),
        },
        _ => Err("Custom types must be specified as simple idents"),
    }
    .map_err(|msg| syn::Error::new(custom_type.span(), msg))?;

    let mod_path = mod_path()?;
    let (impl_spec, derive_ffi_traits) = if options.remote.is_some() {
        (
            quote! { unsafe impl ::uniffi::FfiConverter<crate::UniFfiTag> for #custom_type },
            quote! { ::uniffi::derive_ffi_traits!(local #custom_type); },
        )
    } else {
        (
            quote! { unsafe impl<UT> ::uniffi::FfiConverter<UT> for #custom_type },
            quote! { ::uniffi::derive_ffi_traits!(blanket #custom_type); },
        )
    };

    let (from_custom_param, from_custom_expr) = match options.from_custom {
        Some(convert_closure) => convert_closure.token_tuple(),
        None => (
            quote! { val },
            quote! { <#custom_type as Into<#uniffi_type>>::into(val) },
        ),
    };
    let (try_into_custom_param, try_into_custom_expr) = match options.try_into_custom {
        Some(convert_closure) => convert_closure.token_tuple(),
        None => (
            quote! { val },
            quote! { Ok(<#uniffi_type as TryInto<#custom_type>>::try_into(val)?) },
        ),
    };

    Ok(quote! {
        #[allow(non_camel_case_types)]
        #impl_spec {
            // Note: the UniFFI type needs to implement both `Lower` and `Lift'.  We use the
            // `Lower` trait to get the associated type `FfiType` and const `TYPE_ID_META`.  These
            // can't differ between `Lower` and `Lift`.
            type FfiType = <#uniffi_type as ::uniffi::Lower<crate::UniFfiTag>>::FfiType;
            fn lower(#from_custom_param: #custom_type ) -> Self::FfiType {
                <#uniffi_type as ::uniffi::Lower<crate::UniFfiTag>>::lower(#from_custom_expr)
            }

            fn try_lift(v: Self::FfiType) -> ::uniffi::Result<#custom_type> {
                let #try_into_custom_param = <#uniffi_type as ::uniffi::Lift<crate::UniFfiTag>>::try_lift(v)?;
                #try_into_custom_expr
            }

            fn write(#from_custom_param: #custom_type, buf: &mut Vec<u8>) {
                <#uniffi_type as ::uniffi::Lower<crate::UniFfiTag>>::write(#from_custom_expr, buf);
            }

            fn try_read(buf: &mut &[u8]) -> ::uniffi::Result<#custom_type> {
                let #try_into_custom_param = <#uniffi_type as ::uniffi::Lift<crate::UniFfiTag>>::try_read(buf)?;
                #try_into_custom_expr
            }

            const TYPE_ID_META: ::uniffi::MetadataBuffer = ::uniffi::MetadataBuffer::from_code(::uniffi::metadata::codes::TYPE_CUSTOM)
                .concat_str(#mod_path)
                .concat_str(#name)
                .concat(<#uniffi_type as ::uniffi::TypeId<crate::UniFfiTag>>::TYPE_ID_META);
        }

        #derive_ffi_traits
    })
}

pub struct CustomNewtypeArgs {
    ident: Type,
    uniffi_type: Type,
}

impl Parse for CustomNewtypeArgs {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let ident = input.parse()?;
        input.parse::<Token![,]>()?;
        let uniffi_type = input.parse()?;
        Ok(Self { ident, uniffi_type })
    }
}

// Generate TypeConverter implementation for a newtype
pub(crate) fn expand_custom_newtype(args: CustomNewtypeArgs) -> syn::Result<TokenStream> {
    let CustomNewtypeArgs { ident, uniffi_type } = args;

    Ok(quote! {
        uniffi::custom_type!(#ident, #uniffi_type, {
            from_custom: |obj| obj.0,
            try_into_custom: |val| Ok(#ident(val)),
        });
    })
}
