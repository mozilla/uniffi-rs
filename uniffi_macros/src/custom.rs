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
    into_existing: Option<ConvertClosure>,
    try_from_existing: Option<ConvertClosure>,
    remote: Option<kw::remote>,
}

impl CustomTypeOptions {
    fn is_remote(&self) -> bool {
        // FIXME(#2807): Force all custom types to act as if `remote` was present.  This causes the
        // generate code to only implement FfiConverter for the local tag, which is the current way
        // of doing things.
        true
    }
}

impl Parse for CustomTypeOptions {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        parse_comma_separated(input)
    }
}

impl UniffiAttributeArgs for CustomTypeOptions {
    fn parse_one(input: ParseStream<'_>) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::into_existing) {
            input.parse::<kw::into_existing>()?;
            input.parse::<Token![:]>()?;
            Ok(Self {
                into_existing: Some(input.parse()?),
                ..Self::default()
            })
        } else if lookahead.peek(kw::try_from_existing) {
            input.parse::<kw::try_from_existing>()?;
            input.parse::<Token![:]>()?;
            Ok(Self {
                try_from_existing: Some(input.parse()?),
                ..Self::default()
            })
        // Temporarily disabled until we land #2807 and get the new remote types system
        // } else if lookahead.peek(kw::remote) {
        //     Ok(Self {
        //         remote: Some(input.parse()?),
        //         ..Self::default()
        //     })
        } else {
            Err(lookahead.error())
        }
    }

    fn merge(self, other: Self) -> syn::Result<Self> {
        Ok(Self {
            into_existing: either_attribute_arg(self.into_existing, other.into_existing)?,
            try_from_existing: either_attribute_arg(
                self.try_from_existing,
                other.try_from_existing,
            )?,
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
    let (impl_spec, derive_ffi_traits) = if options.is_remote() {
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

    let (into_existing_param, into_existing_expr) = match options.into_existing {
        Some(convert_closure) => convert_closure.token_tuple(),
        None => (
            quote! { val },
            quote! { <#custom_type as Into<#uniffi_type>>::into(val) },
        ),
    };
    let (try_from_existing, try_from_existing_expr) = match options.try_from_existing {
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
            fn lower(#into_existing_param: #custom_type ) -> Self::FfiType {
                <#uniffi_type as ::uniffi::Lower<crate::UniFfiTag>>::lower(#into_existing_expr)
            }

            fn try_lift(v: Self::FfiType) -> ::uniffi::Result<#custom_type> {
                let #try_from_existing = <#uniffi_type as ::uniffi::Lift<crate::UniFfiTag>>::try_lift(v)?;
                #try_from_existing_expr
            }

            fn write(#into_existing_param: #custom_type, buf: &mut Vec<u8>) {
                <#uniffi_type as ::uniffi::Lower<crate::UniFfiTag>>::write(#into_existing_expr, buf);
            }

            fn try_read(buf: &mut &[u8]) -> ::uniffi::Result<#custom_type> {
                let #try_from_existing = <#uniffi_type as ::uniffi::Lift<crate::UniFfiTag>>::try_read(buf)?;
                #try_from_existing_expr
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
            into_existing: |obj| obj.0,
            try_from_existing: |val| Ok(#ident(val)),
        });
    })
}
