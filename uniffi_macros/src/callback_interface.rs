/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::{
    fnsig::{FnKind, FnSignature},
    util::{create_metadata_items, ident_to_string, mod_path, syn_err, ArgumentNotAllowedHere},
};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use std::iter;
use syn::{ext::IdentExt, Ident, ItemTrait, TraitItem};

pub(crate) fn expand_callback_interface(
    trait_: ItemTrait,
    _args: ArgumentNotAllowedHere,
) -> syn::Result<TokenStream> {
    let item = CallbackInterfaceItem::new(trait_)?;
    let trait_name = &item.name;
    let trait_impl_ident = Ident::new(
        &format!("UniFFICallbackHandler{trait_name}"),
        Span::call_site(),
    );
    let internals_ident = Ident::new(
        &format!(
            "UNIFFI_FOREIGN_CALLBACK_INTERNALS_{}",
            trait_name.to_ascii_uppercase()
        ),
        Span::call_site(),
    );
    let mod_path = mod_path()?;
    let trait_impl = callback_interface_trait_impl(&item, &trait_impl_ident, &internals_ident)?;
    let metadata_items = callback_interface_metadata_items(&item, &mod_path)
        .unwrap_or_else(|e| vec![syn::Error::into_compile_error(e)]);
    let init_ident = Ident::new(
        &uniffi_meta::init_callback_fn_symbol_name(&mod_path, &item.name),
        Span::call_site(),
    );

    Ok(quote! {
        #[doc(hidden)]
        static #internals_ident: ::uniffi::ForeignCallbackInternals = ::uniffi::ForeignCallbackInternals::new();

        #[doc(hidden)]
        #[no_mangle]
        pub extern "C" fn #init_ident(callback: ::uniffi::ForeignCallback, _: &mut ::uniffi::RustCallStatus) {
            #internals_ident.set_callback(callback);
        }

        #trait_impl

        #(#metadata_items)*
    })
}

/// Stores data from a parsed callback interface trait
struct CallbackInterfaceItem {
    ident: Ident,
    name: String,
    methods: Vec<FnSignature>,
}

impl CallbackInterfaceItem {
    fn new(trait_: ItemTrait) -> syn::Result<Self> {
        let ident = trait_.ident;
        let name = ident.unraw().to_string();
        let methods = trait_
            .items
            .into_iter()
            .enumerate()
            .map(|(i, trait_item)| match trait_item {
                TraitItem::Fn(method) => Ok(FnSignature::new_callback_interface_method(
                    ident.clone(),
                    i as u32,
                    method.sig,
                )?),
                TraitItem::Const(_) => syn_err(
                    trait_item,
                    "associated constants are not supported in callback interfaces",
                ),
                TraitItem::Type(_) => syn_err(
                    trait_item,
                    "associated types are not supported in callback interfaces",
                ),
                TraitItem::Macro(_) => syn_err(
                    trait_item,
                    "macros are not supported in callback interfaces",
                ),
                _ => syn_err(trait_item, "unknown item type in callback interface"),
            })
            .collect::<syn::Result<Vec<_>>>()?;
        Ok(Self {
            ident,
            name,
            methods,
        })
    }
}

// Register a foreign callback for getting across the FFI.

fn callback_interface_trait_impl(
    item: &CallbackInterfaceItem,
    ident: &Ident,
    internals_ident: &Ident,
) -> syn::Result<TokenStream> {
    let trait_ident = &item.ident;
    let trait_name = &item.name;
    let trait_impl_methods = item
        .methods
        .iter()
        .map(|m| gen_method_impl(m, internals_ident))
        .collect::<syn::Result<Vec<_>>>()?;

    Ok(quote! {
        #[doc(hidden)]
        #[derive(Debug)]
        struct #ident {
            handle: u64,
        }

        impl #ident {
            fn new(handle: u64) -> Self {
                Self { handle }
            }
        }

        impl Drop for #ident {
            fn drop(&mut self) {
                #internals_ident.invoke_callback::<(), crate::UniFfiTag>(
                    self.handle, uniffi::IDX_CALLBACK_FREE, Default::default()
                )
            }
        }

        ::uniffi::deps::static_assertions::assert_impl_all!(#ident: Send);

        impl #trait_ident for #ident {
            #(#trait_impl_methods)*
        }

        ::uniffi::ffi_converter_callback_interface!(#trait_ident, #ident, #trait_name, crate::UniFfiTag);
    })
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
        FnKind::CallbackInterfaceMethod { index, .. } => index + 1,
        k => {
            return Err(syn::Error::new(
                sig.span,
                format!(
                    "Internal UniFFI error: Unexpected function kind for callback interface {k:?}"
                ),
            ));
        }
    };

    if receiver.is_none() {
        return Err(syn::Error::new(
            sig.span,
            "callback interface methods must take &self as their first argument",
        ));
    }
    let params = sig.params();
    let buf_ident = Ident::new("uniffi_args_buf", Span::call_site());
    let mut write_exprs = sig.write_exprs(&buf_ident).peekable();

    let construct_args_buf = if write_exprs.peek().is_some() {
        quote! {
            let mut #buf_ident = ::std::vec::Vec::new();
            #(#write_exprs;)*
        }
    } else {
        quote! { let #buf_ident = ::std::vec::Vec::new(); }
    };

    Ok(quote! {
        fn #ident(&self, #(#params),*) -> #return_ty {
            #construct_args_buf
            let uniffi_args_rbuf = uniffi::RustBuffer::from_vec(#buf_ident);

            #internals_ident.invoke_callback::<#return_ty, crate::UniFfiTag>(self.handle, #index, uniffi_args_rbuf)
        }
    })
}

fn callback_interface_metadata_items(
    item: &CallbackInterfaceItem,
    module_path: &str,
) -> syn::Result<Vec<TokenStream>> {
    let trait_name = ident_to_string(&item.ident);
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
        .chain(item.methods.iter().map(FnSignature::metadata_items))
        .collect()
}
