/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::util::{
    either_attribute_arg, ident_to_string, mod_path, parse_comma_separated, tagged_impl_header,
    AttributeSliceExt, UniffiAttributeArgs,
};
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    Data, DeriveInput, Path, Token,
};

pub fn expand_custom(input: DeriveInput) -> syn::Result<TokenStream> {
    if !matches!(input.data, Data::Struct(_)) {
        return Err(syn::Error::new(
            Span::call_site(),
            "This derive must only be used on structs",
        ));
    };

    let ident = &input.ident;
    let attr = input.attrs.parse_uniffi_attr_args::<CustomTypeAttr>()?;

    let ffi_converter = custom_ffi_converter_impl(ident, &attr)?;

    let type_converter = match attr.newtype {
        Some(builtin) => custom_ffi_type_converter(ident, &builtin)?,
        None => TokenStream::new(),
    };
    Ok(quote! {
        #ffi_converter
        #type_converter
    })
}

pub(crate) fn expand_ffi_converter_custom_type(
    attr: CustomTypeAttr,
    input: DeriveInput,
) -> syn::Result<TokenStream> {
    custom_ffi_converter_impl(&input.ident, &attr)
}

// For custom scaffolding types we need to generate an FfiConverter impl based on the
// UniffiCustomTypeConverter implementation that the library supplies
pub(crate) fn custom_ffi_converter_impl(
    ident: &Ident,
    attr: &CustomTypeAttr,
) -> syn::Result<TokenStream> {
    if attr.builtin.is_some() && attr.newtype.is_some() {
        return Err(syn::Error::new(
            Span::call_site(),
            "Custom types must not specify both `builtin` and `newtype`",
        ));
    }
    let Some(builtin) = attr.builtin.as_ref().or(attr.newtype.as_ref()) else {
        return Err(syn::Error::new(
            Span::call_site(),
            "Custom types must specify the builtin/newtype",
        ));
    };
    let ffi_name = match builtin.to_string().as_str() {
        "i8" | "u8" | "i16" | "u16" | "i32" | "u32" | "i64" | "u64" | "f32" | "f64" => {
            builtin.to_string()
        }
        "String" => "::uniffi::RustBuffer".to_string(),
        // in theory we could support others, but for now...
        _ => {
            return Err(syn::Error::new(
                Span::call_site(),
                "Type is not supported as a custom type builtin",
            ));
        }
    };

    let ffi_path: TokenStream = ffi_name.parse()?;

    let impl_spec = tagged_impl_header("FfiConverter", ident, attr.tag.as_ref());
    let name = ident_to_string(ident);
    let mod_path = mod_path()?;

    Ok(quote! {
        #[automatically_derived]
        unsafe #impl_spec {
            type FfiType = #ffi_path;
            fn lower(obj: #ident ) -> Self::FfiType {
                <#builtin as ::uniffi::FfiConverter<crate::UniFfiTag>>::lower(<#ident as UniffiCustomTypeConverter>::from_custom(obj))
            }

            fn try_lift(v: Self::FfiType) -> uniffi::Result<#ident> {
                <#ident as UniffiCustomTypeConverter>::into_custom(<#builtin as ::uniffi::FfiConverter<crate::UniFfiTag>>::try_lift(v)?)
            }

            fn write(obj: #ident, buf: &mut Vec<u8>) {
                <#builtin as ::uniffi::FfiConverter<crate::UniFfiTag>>::write(<#ident as UniffiCustomTypeConverter>::from_custom(obj), buf);
            }

            fn try_read(buf: &mut &[u8]) -> uniffi::Result<#ident> {
                <#ident as UniffiCustomTypeConverter>::into_custom(<#builtin as ::uniffi::FfiConverter<crate::UniFfiTag>>::try_read(buf)?)
            }

            ::uniffi::ffi_converter_default_return!(crate::UniFfiTag);

            const TYPE_ID_META: ::uniffi::MetadataBuffer = ::uniffi::MetadataBuffer::from_code(::uniffi::metadata::codes::TYPE_CUSTOM)
                .concat_str(#mod_path)
                .concat_str(#name)
                .concat(<#builtin as ::uniffi::FfiConverter<crate::UniFfiTag>>::TYPE_ID_META);
        }
    })
}

fn custom_ffi_type_converter(ident: &Ident, builtin: &Ident) -> syn::Result<TokenStream> {
    Ok(quote! {
        impl UniffiCustomTypeConverter for #ident {
            type Builtin = #builtin;

            fn into_custom(val: Self::Builtin) -> uniffi::Result<Self> {
                Ok(#ident(val))
            }

            fn from_custom(obj: Self) -> Self::Builtin {
                obj.0
            }
        }
    })
}

mod kw {
    syn::custom_keyword!(tag);
    syn::custom_keyword!(builtin);
    syn::custom_keyword!(newtype);
}

#[derive(Default)]
pub(crate) struct CustomTypeAttr {
    tag: Option<Path>,
    builtin: Option<Ident>,
    newtype: Option<Ident>,
}

impl UniffiAttributeArgs for CustomTypeAttr {
    fn parse_one(input: ParseStream<'_>) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::tag) {
            let _: kw::tag = input.parse()?;
            let _: Token![=] = input.parse()?;
            Ok(Self {
                tag: Some(input.parse()?),
                ..Self::default()
            })
        } else if lookahead.peek(kw::builtin) {
            let _: kw::builtin = input.parse()?;
            let _: Token![=] = input.parse()?;
            Ok(Self {
                builtin: Some(input.parse()?),
                ..Self::default()
            })
        } else if lookahead.peek(kw::newtype) {
            let _: kw::newtype = input.parse()?;
            let _: Token![=] = input.parse()?;
            Ok(Self {
                newtype: Some(input.parse()?),
                ..Self::default()
            })
        } else {
            Err(lookahead.error())
        }
    }

    fn merge(self, other: Self) -> syn::Result<Self> {
        Ok(Self {
            tag: either_attribute_arg(self.tag, other.tag)?,
            builtin: either_attribute_arg(self.builtin, other.builtin)?,
            newtype: either_attribute_arg(self.newtype, other.newtype)?,
        })
    }
}

impl Parse for CustomTypeAttr {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        parse_comma_separated(input)
    }
}
