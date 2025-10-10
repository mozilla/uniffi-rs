/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::util::{
    create_metadata_items, either_attribute_arg, ident_to_string, kw, parse_comma_separated,
    UniffiAttributeArgs,
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
    lower: Option<ConvertClosure>,
    try_lift: Option<ConvertClosure>,
    remote: Option<kw::remote>,
}

impl CustomTypeOptions {
    fn is_remote(&self) -> bool {
        self.remote.is_some()
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
        if lookahead.peek(kw::lower) {
            input.parse::<kw::lower>()?;
            input.parse::<Token![:]>()?;
            Ok(Self {
                lower: Some(input.parse()?),
                ..Self::default()
            })
        } else if lookahead.peek(kw::try_lift) {
            input.parse::<kw::try_lift>()?;
            input.parse::<Token![:]>()?;
            Ok(Self {
                try_lift: Some(input.parse()?),
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
            lower: either_attribute_arg(self.lower, other.lower)?,
            try_lift: either_attribute_arg(self.try_lift, other.try_lift)?,
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

    let (lower_param, lower_expr) = match options.lower {
        Some(convert_closure) => convert_closure.token_tuple(),
        None => (
            quote! { val },
            quote! { <#custom_type as Into<#uniffi_type>>::into(val) },
        ),
    };
    let (try_lift, try_lift_expr) = match options.try_lift {
        Some(convert_closure) => convert_closure.token_tuple(),
        None => (
            quote! { val },
            quote! { ::core::result::Result::Ok(<#uniffi_type as TryInto<#custom_type>>::try_into(val)?) },
        ),
    };

    let build_context = quote::quote! {
        || {
            let location = ::core::panic::Location::caller();
            ::std::format!(
                "Lifting custom type `{}` from FFI type `{}` failed at {}:{}",
                ::core::any::type_name::<#custom_type>(),
                ::core::any::type_name::<#uniffi_type>(),
                location.file(),
                location.line()
            )
        }
    };

    // We allow needless_question_mark here because the error type of `TryInto` only needs conversion
    // iff it is not a `anyhow::Error` already.
    let try_lift_expr = quote::quote! {
        #[allow(clippy::needless_question_mark)]
        {
            ::uniffi::deps::anyhow::Context::with_context(
                (move || -> ::uniffi::Result<#custom_type> { #try_lift_expr })(),
                #build_context
            )
        }
    };

    let docstring = ""; // todo...
    let meta_static_var = custom_type_meta_static_var(&name, &uniffi_type, docstring)?;

    Ok(quote! {
        #[allow(non_camel_case_types)]
        #impl_spec {
            // Note: the UniFFI type needs to implement both `Lower` and `Lift'.  We use the
            // `Lower` trait to get the associated type `FfiType` and const `TYPE_ID_META`.  These
            // can't differ between `Lower` and `Lift`.
            type FfiType = <#uniffi_type as ::uniffi::Lower<crate::UniFfiTag>>::FfiType;
            fn lower(#lower_param: #custom_type ) -> Self::FfiType {
                <#uniffi_type as ::uniffi::Lower<crate::UniFfiTag>>::lower(#lower_expr)
            }

            fn try_lift(v: Self::FfiType) -> ::uniffi::Result<#custom_type> {
                let #try_lift = <#uniffi_type as ::uniffi::Lift<crate::UniFfiTag>>::try_lift(v)?;
                #try_lift_expr
            }

            fn write(#lower_param: #custom_type, buf: &mut Vec<u8>) {
                <#uniffi_type as ::uniffi::Lower<crate::UniFfiTag>>::write(#lower_expr, buf);
            }

            fn try_read(buf: &mut &[u8]) -> ::uniffi::Result<#custom_type> {
                let #try_lift = <#uniffi_type as ::uniffi::Lift<crate::UniFfiTag>>::try_read(buf)?;
                #try_lift_expr
            }

            const TYPE_ID_META: ::uniffi::MetadataBuffer = ::uniffi::MetadataBuffer::from_code(::uniffi::metadata::codes::TYPE_CUSTOM)
                .concat_str(module_path!())
                .concat_str(#name)
                .concat(<#uniffi_type as ::uniffi::TypeId<crate::UniFfiTag>>::TYPE_ID_META);
        }

        #derive_ffi_traits

        #meta_static_var
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
            lower: |obj| obj.0,
            try_lift: |val| Ok(#ident(val)),
        });
    })
}

pub(crate) fn custom_type_meta_static_var(
    name: &str,
    builtin: &Type,
    docstring: &str,
) -> syn::Result<TokenStream> {
    let builtin = crate::ffiops::type_id_meta(builtin);
    let metadata_expr = quote! {
            ::uniffi::MetadataBuffer::from_code(::uniffi::metadata::codes::CUSTOM_TYPE)
                .concat_str(module_path!())
                .concat_str(#name)
                .concat(#builtin)
                .concat_long_str(#docstring)
    };
    Ok(create_metadata_items(
        "custom_type",
        name,
        metadata_expr,
        None,
    ))
}
