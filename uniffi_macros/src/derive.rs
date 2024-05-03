/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! General handling for the derive and udl_derive macros

use crate::util::kw;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    DeriveInput,
};

pub fn expand_derive(
    kind: DeriveKind,
    input: DeriveInput,
    options: DeriveOptions,
) -> syn::Result<TokenStream> {
    match kind {
        DeriveKind::Record(_) => crate::record::expand_record(input, options),
        DeriveKind::Object(_) => crate::object::expand_object(input, options),
        DeriveKind::Enum(_) => crate::enum_::expand_enum(input, options),
        DeriveKind::Error(_) => crate::error::expand_error(input, options),
    }
}

pub enum DeriveKind {
    Record(kw::Record),
    Enum(kw::Enum),
    Error(kw::Error),
    Object(kw::Object),
}

impl Parse for DeriveKind {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::Record) {
            Ok(Self::Record(input.parse()?))
        } else if lookahead.peek(kw::Enum) {
            Ok(Self::Enum(input.parse()?))
        } else if lookahead.peek(kw::Error) {
            Ok(Self::Error(input.parse()?))
        } else if lookahead.peek(kw::Object) {
            Ok(Self::Object(input.parse()?))
        } else {
            Err(lookahead.error())
        }
    }
}

pub struct DeriveOptions {
    /// Should we implement FFI traits for the local UniFfiTag only?
    pub local_tag: bool,
    /// Should we generate metadata symbols?
    pub generate_metadata: bool,
}

/// default() is used to construct a DeriveOptions for a regular `derive` invocation
impl Default for DeriveOptions {
    fn default() -> Self {
        Self {
            local_tag: false,
            generate_metadata: true,
        }
    }
}

impl DeriveOptions {
    /// Construct DeriveOptions for `udl_derive`
    pub fn udl_derive() -> Self {
        Self {
            // TODO: change this to false
            local_tag: true,
            generate_metadata: false,
        }
    }

    /// DeriveOptions for `#[remote]`
    pub fn remote() -> Self {
        Self {
            local_tag: true,
            generate_metadata: true,
        }
    }

    /// DeriveOptions for `#[udl_remote]`
    pub fn udl_remote() -> Self {
        Self {
            local_tag: true,
            generate_metadata: false,
        }
    }

    /// Generate the impl header for a FFI trait
    ///
    /// This will output something like `impl<UT> FfiConverter<UT> for #type`.  The caller is
    /// responsible for providing the body if the impl block.
    pub fn ffi_impl_header(&self, trait_name: &str, ident: &impl ToTokens) -> TokenStream {
        let trait_name = Ident::new(trait_name, Span::call_site());
        if self.local_tag {
            quote! { impl ::uniffi::#trait_name<crate::UniFfiTag> for #ident }
        } else {
            quote! { impl<UT> ::uniffi::#trait_name<UT> for #ident }
        }
    }

    /// Generate a call to `derive_ffi_traits!` that will derive all the FFI traits
    pub fn derive_all_ffi_traits(&self, ty: &Ident) -> TokenStream {
        if self.local_tag {
            quote! { ::uniffi::derive_ffi_traits!(local #ty); }
        } else {
            quote! { ::uniffi::derive_ffi_traits!(blanket #ty); }
        }
    }

    /// Generate a call to `derive_ffi_traits!` that will derive some of the FFI traits
    pub fn derive_ffi_traits(&self, ty: impl ToTokens, trait_names: &[&str]) -> TokenStream {
        let trait_idents = trait_names
            .iter()
            .map(|name| Ident::new(name, Span::call_site()));
        if self.local_tag {
            quote! {
                #(
                    ::uniffi::derive_ffi_traits!(impl #trait_idents<crate::UniFfiTag> for #ty);
                )*
            }
        } else {
            quote! {
                #(
                    ::uniffi::derive_ffi_traits!(impl<UT> #trait_idents<UT> for #ty);
                )*
            }
        }
    }
}
