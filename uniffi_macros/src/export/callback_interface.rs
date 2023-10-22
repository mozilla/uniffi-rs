/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::{
    export::ImplItem,
    fnsig::{FnKind, FnSignature, ReceiverArg},
    util::{create_metadata_items, ident_to_string, mod_path, tagged_impl_header},
};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use std::iter;
use syn::Ident;

pub(super) fn trait_impl(
    mod_path: &str,
    trait_ident: &Ident,
    items: &[ImplItem],
    trait_interface: bool,
) -> syn::Result<TokenStream> {
    let trait_name = ident_to_string(trait_ident);
    let trait_impl_ident = trait_impl_ident(&trait_name);
    let internals_ident = internals_ident(&trait_name);
    let init_ident = Ident::new(
        &uniffi_meta::init_callback_fn_symbol_name(mod_path, &trait_name),
        Span::call_site(),
    );

    let trait_impl_methods = items
        .iter()
        .map(|item| match item {
            ImplItem::Method(sig) => gen_method_impl(sig, &internals_ident),
            _ => unreachable!("traits have no constructors"),
        })
        .collect::<syn::Result<TokenStream>>()?;

    let uniffi_foreign_handle = trait_interface.then(|| {
        quote! {
            fn uniffi_foreign_handle(&self) -> Option<::uniffi::Handle> {
                let raw_clone = #internals_ident.invoke_callback::<u64, crate::UniFfiTag>(
                    self.handle, uniffi::IDX_CALLBACK_CLONE, Default::default()
                );
                let handle = ::uniffi::Handle::from_raw(raw_clone)
                    .unwrap_or_else(|| panic!("{} IDX_CALLBACK_CLONE returned null handle", #trait_name));
                Some(handle)
            }
        }
    });

    Ok(quote! {
        #[doc(hidden)]
        static #internals_ident: ::uniffi::ForeignCallbackInternals = ::uniffi::ForeignCallbackInternals::new();

        #[doc(hidden)]
        #[no_mangle]
        pub extern "C" fn #init_ident(callback: ::uniffi::ForeignCallback) {
            #internals_ident.set_callback(callback);
        }

        #[doc(hidden)]
        #[derive(Debug)]
        struct #trait_impl_ident {
            handle: ::uniffi::Handle,
        }

        impl #trait_impl_ident {
            fn new(handle: ::uniffi::Handle) -> Self {
                Self { handle }
            }
        }

        impl ::std::ops::Drop for #trait_impl_ident {
            fn drop(&mut self) {
                #internals_ident.invoke_callback::<(), crate::UniFfiTag>(
                    self.handle, uniffi::IDX_CALLBACK_FREE, Default::default()
                )
            }
        }

        ::uniffi::deps::static_assertions::assert_impl_all!(#trait_impl_ident: ::core::marker::Send);

        impl #trait_ident for #trait_impl_ident {
            #trait_impl_methods
            #uniffi_foreign_handle
        }
    })
}

pub fn trait_impl_ident(trait_name: &str) -> Ident {
    Ident::new(
        &format!("UniFFICallbackHandler{trait_name}"),
        Span::call_site(),
    )
}

pub fn internals_ident(trait_name: &str) -> Ident {
    Ident::new(
        &format!(
            "UNIFFI_FOREIGN_CALLBACK_INTERNALS_{}",
            trait_name.to_ascii_uppercase()
        ),
        Span::call_site(),
    )
}

pub fn ffi_converter_callback_interface_impl(
    trait_ident: &Ident,
    trait_impl_ident: &Ident,
    udl_mode: bool,
) -> TokenStream {
    let trait_name = ident_to_string(trait_ident);
    let dyn_trait = quote! { dyn #trait_ident };
    let box_dyn_trait = quote! { ::std::boxed::Box<#dyn_trait> };
    let lift_impl_spec = tagged_impl_header("Lift", &box_dyn_trait, udl_mode);
    let lift_ref_impl_spec = tagged_impl_header("LiftRef", &dyn_trait, udl_mode);
    let mod_path = match mod_path() {
        Ok(p) => p,
        Err(e) => return e.into_compile_error(),
    };

    quote! {
        #[doc(hidden)]
        #[automatically_derived]
        unsafe #lift_impl_spec {
            type FfiType = u64;

            fn try_lift(v: Self::FfiType) -> ::uniffi::deps::anyhow::Result<Self> {
                let handle = match ::uniffi::Handle::from_raw(v) {
                    Some(h) => h,
                    None => ::uniffi::deps::anyhow::bail!("{}::try_lift: null handle", #trait_name),
                };
                Ok(::std::boxed::Box::new(<#trait_impl_ident>::new(handle)))
            }

            fn try_read(buf: &mut &[u8]) -> ::uniffi::deps::anyhow::Result<Self> {
                use uniffi::deps::bytes::Buf;
                ::uniffi::check_remaining(buf, 8)?;
                <Self as ::uniffi::Lift<crate::UniFfiTag>>::try_lift(buf.get_u64())
            }

            const TYPE_ID_META: ::uniffi::MetadataBuffer = ::uniffi::MetadataBuffer::from_code(
                ::uniffi::metadata::codes::TYPE_CALLBACK_INTERFACE,
            )
            .concat_str(#mod_path)
            .concat_str(#trait_name);
        }

        unsafe #lift_ref_impl_spec {
            type LiftType = #box_dyn_trait;
        }
    }
}

fn gen_method_impl(sig: &FnSignature, internals_ident: &Ident) -> syn::Result<TokenStream> {
    let FnSignature {
        ident,
        return_ty,
        kind,
        receiver,
        ..
    } = sig;
    let index = match kind {
        // Note: the callback index is 1-based, since 0 is reserved for the free function
        FnKind::TraitMethod { index, .. } => index + 1,
        k => {
            return Err(syn::Error::new(
                sig.span,
                format!(
                    "Internal UniFFI error: Unexpected function kind for callback interface {k:?}"
                ),
            ));
        }
    };

    let self_param = match receiver {
        None => {
            return Err(syn::Error::new(
                sig.span,
                "callback interface methods must take &self as their first argument",
            ));
        }
        Some(ReceiverArg::Ref) => quote! { &self },
        Some(ReceiverArg::Arc) => quote! { self: Arc<Self> },
    };
    let params = sig.params();
    let buf_ident = Ident::new("uniffi_args_buf", Span::call_site());
    let write_exprs = sig.write_exprs(&buf_ident);

    Ok(quote! {
        fn #ident(#self_param, #(#params),*) -> #return_ty {
            #[allow(unused_mut)]
            let mut #buf_ident = ::std::vec::Vec::new();
            #(#write_exprs;)*
            let uniffi_args_rbuf = uniffi::RustBuffer::from_vec(#buf_ident);

            #internals_ident.invoke_callback::<#return_ty, crate::UniFfiTag>(self.handle, #index, uniffi_args_rbuf)
        }
    })
}

pub(super) fn metadata_items(
    self_ident: &Ident,
    items: &[ImplItem],
    module_path: &str,
) -> syn::Result<Vec<TokenStream>> {
    let trait_name = ident_to_string(self_ident);
    let callback_interface_items = create_metadata_items(
        "callback_interface",
        &trait_name,
        quote! {
            ::uniffi::MetadataBuffer::from_code(::uniffi::metadata::codes::CALLBACK_INTERFACE)
                .concat_str(#module_path)
                .concat_str(#trait_name)
        },
        None,
    );

    iter::once(Ok(callback_interface_items))
        .chain(items.iter().map(|item| match item {
            ImplItem::Method(sig) => sig.metadata_items(),
            _ => unreachable!("traits have no constructors"),
        }))
        .collect()
}
