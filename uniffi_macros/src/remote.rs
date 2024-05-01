/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    Ident, Token, Type,
};

pub struct RemoteTypeArgs {
    pub ty: Type,
    pub sep: Token![,],
    pub implementing_crate: Ident,
}

impl Parse for RemoteTypeArgs {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        Ok(Self {
            ty: input.parse()?,
            sep: input.parse()?,
            implementing_crate: input.parse()?,
        })
    }
}

pub fn expand_remote_type(args: RemoteTypeArgs) -> TokenStream {
    let RemoteTypeArgs {
        ty,
        implementing_crate,
        ..
    } = args;
    let existing_tag = quote! { #implementing_crate::UniFfiTag };

    quote! {
        unsafe impl ::uniffi::FfiConverter<crate::UniFfiTag> for #ty {
            type FfiType = <#ty as ::uniffi::FfiConverter<#existing_tag>>::FfiType;

            fn lower(obj: #ty) -> Self::FfiType {
                <#ty as ::uniffi::FfiConverter<#existing_tag>>::lower(obj)
            }

            fn try_lift(v: Self::FfiType) -> ::uniffi::Result<#ty> {
                <#ty as ::uniffi::FfiConverter<#existing_tag>>::try_lift(v)
            }

            fn write(obj: #ty, buf: &mut Vec<u8>) {
                <#ty as ::uniffi::FfiConverter<#existing_tag>>::write(obj, buf)
            }

            fn try_read(buf: &mut &[u8]) -> ::uniffi::Result<#ty> {
                <#ty as ::uniffi::FfiConverter<#existing_tag>>::try_read(buf)
            }

            const TYPE_ID_META: ::uniffi::MetadataBuffer =
                <#ty as ::uniffi::FfiConverter<#existing_tag>>::TYPE_ID_META;
        }

        ::uniffi::derive_ffi_traits!(local #ty);
    }
}
