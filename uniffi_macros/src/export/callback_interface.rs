/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::{
    export::ImplItem,
    ffiops,
    fnsig::{FnKind, FnSignature, ReceiverArg},
    util::{
        async_trait_annotation, create_metadata_items, derive_ffi_traits, ident_to_string,
        mod_path, tagged_impl_header, wasm_single_threaded_annotation,
    },
};
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use std::iter;
use syn::Ident;

/// Generate a trait impl that calls foreign callbacks
///
/// This generates:
///    * A `repr(C)` VTable struct where each field is the FFI function for the trait method.
///    * A FFI function for foreign code to set their VTable for the interface
///    * An implementation of the trait using that VTable
pub(super) fn trait_impl(
    mod_path: &str,
    trait_ident: &Ident,
    items: &[ImplItem],
) -> syn::Result<TokenStream> {
    let trait_name = ident_to_string(trait_ident);
    let trait_impl_ident = trait_impl_ident(&trait_name);
    let vtable_type = format_ident!("UniFfiTraitVtable{trait_name}");
    let vtable_cell = format_ident!("UNIFFI_TRAIT_CELL_{}", trait_name.to_uppercase());
    let init_ident = Ident::new(
        &uniffi_meta::init_callback_vtable_fn_symbol_name(mod_path, &trait_name),
        Span::call_site(),
    );
    let methods = items
        .iter()
        .map(|item| match item {
            ImplItem::Constructor(sig) => Err(syn::Error::new(
                sig.span,
                "Constructors not allowed in trait interfaces",
            )),
            ImplItem::Method(sig) => Ok(sig),
        })
        .collect::<syn::Result<Vec<_>>>()?;

    let vtable_fields = methods.iter().map(|sig| {
        let ident = &sig.ident;
        let param_names = sig.scaffolding_param_names();
        let param_types = sig.scaffolding_param_types();
        let lift_return_type = ffiops::lift_return_type(&sig.return_ty);
        if !sig.is_async {
            quote! {
                pub #ident: extern "C" fn(
                    uniffi_handle: u64,
                    #(#param_names: #param_types,)*
                    uniffi_out_return: &mut #lift_return_type,
                    uniffi_out_call_status: &mut ::uniffi::RustCallStatus,
                ),
            }
        } else {
            quote! {
                pub #ident: extern "C" fn(
                    uniffi_handle: u64,
                    #(#param_names: #param_types,)*
                    uniffi_callback: ::uniffi::ForeignFutureCallback<#lift_return_type>,
                    uniffi_callback_data: u64,
                    uniffi_out_dropped_callback: &mut ::uniffi::ForeignFutureDroppedCallbackStruct,
                ),
            }
        }
    });

    let trait_impl_methods = methods
        .iter()
        .map(|sig| gen_method_impl(sig, &vtable_cell))
        .collect::<syn::Result<Vec<_>>>()?;
    let has_async_method = methods.iter().any(|m| m.is_async);

    // Conditionally apply the async_trait attribute with or without ?Send based on the target
    let impl_attributes = has_async_method.then(async_trait_annotation);

    let single_threaded_annotation = wasm_single_threaded_annotation();

    Ok(quote! {
        #[allow(missing_docs)]
        pub struct #vtable_type {
            #(#vtable_fields)*
            pub uniffi_free: extern "C" fn(handle: u64),
        }

        static #vtable_cell: ::uniffi::UniffiForeignPointerCell::<#vtable_type> = ::uniffi::UniffiForeignPointerCell::<#vtable_type>::new();

        #[allow(missing_docs)]
        #[unsafe(no_mangle)]
        pub extern "C" fn #init_ident(vtable: ::std::ptr::NonNull<#vtable_type>) {
            #vtable_cell.set(vtable);
        }

        #[derive(Debug)]
        struct #trait_impl_ident {
            handle: u64,
        }

        impl #trait_impl_ident {
            fn new(handle: u64) -> Self {
                Self { handle }
            }
        }

        #single_threaded_annotation
        ::uniffi::deps::static_assertions::assert_impl_all!(#trait_impl_ident: ::core::marker::Send);

        #impl_attributes
        impl #trait_ident for #trait_impl_ident {
            #(#trait_impl_methods)*
        }

        impl ::std::ops::Drop for #trait_impl_ident {
            fn drop(&mut self) {
                let vtable = #vtable_cell.get();
                (vtable.uniffi_free)(self.handle);
            }
        }
    })
}

pub fn trait_impl_ident(trait_name: &str) -> Ident {
    Ident::new(
        &format!("UniFFICallbackHandler{trait_name}"),
        Span::call_site(),
    )
}

pub fn ffi_converter_callback_interface_impl(
    trait_ident: &Ident,
    trait_impl_ident: &Ident,
) -> TokenStream {
    // TODO: support remote callback interfaces
    let remote = false;
    let trait_name = ident_to_string(trait_ident);
    let dyn_trait = quote! { dyn #trait_ident };
    let box_dyn_trait = quote! { ::std::boxed::Box<#dyn_trait> };
    let lift_impl_spec = tagged_impl_header("Lift", &box_dyn_trait, remote);
    let type_id_impl_spec = tagged_impl_header("TypeId", &box_dyn_trait, remote);
    let derive_ffi_traits = derive_ffi_traits(&box_dyn_trait, remote, &["LiftRef", "LiftReturn"]);
    let mod_path = match mod_path() {
        Ok(p) => p,
        Err(e) => return e.into_compile_error(),
    };
    let try_lift_self = ffiops::try_lift(quote! { Self });

    quote! {
        #[doc(hidden)]
        #[automatically_derived]
        unsafe #lift_impl_spec {
            type FfiType = u64;

            fn try_lift(v: Self::FfiType) -> ::uniffi::deps::anyhow::Result<Self> {
                ::std::result::Result::Ok(::std::boxed::Box::new(<#trait_impl_ident>::new(v)))
            }

            fn try_read(buf: &mut &[u8]) -> ::uniffi::deps::anyhow::Result<Self> {
                use ::uniffi::deps::bytes::Buf;
                ::uniffi::check_remaining(buf, 8)?;
                #try_lift_self(buf.get_u64())
            }
        }

        #[doc(hidden)]
        #[automatically_derived]
        #type_id_impl_spec {
            const TYPE_ID_META: ::uniffi::MetadataBuffer = ::uniffi::MetadataBuffer::from_code(
                ::uniffi::metadata::codes::TYPE_CALLBACK_INTERFACE,
            )
            .concat_str(#mod_path)
            .concat_str(#trait_name);
        }

        #derive_ffi_traits
    }
}

/// Generate a single method for [trait_impl].  This implements a trait method by invoking a
/// foreign-supplied callback.
fn gen_method_impl(sig: &FnSignature, vtable_cell: &Ident) -> syn::Result<TokenStream> {
    let FnSignature {
        ident,
        is_async,
        return_ty,
        kind,
        receiver,
        name,
        span,
        ..
    } = sig;

    if !matches!(kind, FnKind::TraitMethod { .. }) {
        return Err(syn::Error::new(
            *span,
            format!(
                "Internal UniFFI error: Unexpected function kind for callback interface {name}: {kind:?}",
            ),
        ));
    }

    let self_param = match receiver {
        Some(ReceiverArg::Ref) => quote! { &self },
        Some(ReceiverArg::Arc) => quote! { self: Arc<Self> },
        None => {
            return Err(syn::Error::new(
                *span,
                "callback interface methods must take &self as their first argument",
            ));
        }
    };

    let params = sig.params();
    let lower_exprs = sig.args.iter().map(|a| {
        let lower = ffiops::lower(&a.ty);
        let ident = &a.ident;
        quote! { #lower(#ident) }
    });

    let lift_return_type = ffiops::lift_return_type(&sig.return_ty);
    let lift_foreign_return = ffiops::lift_foreign_return(&sig.return_ty);

    if !is_async {
        Ok(quote! {
            fn #ident(#self_param, #(#params),*) -> #return_ty {
                let vtable = #vtable_cell.get();
                let mut uniffi_call_status: ::uniffi::RustCallStatus = ::std::default::Default::default();
                let mut uniffi_return_value: #lift_return_type = ::uniffi::FfiDefault::ffi_default();
                (vtable.#ident)(self.handle, #(#lower_exprs,)* &mut uniffi_return_value, &mut uniffi_call_status);
                #lift_foreign_return(uniffi_return_value, uniffi_call_status)
            }
        })
    } else {
        Ok(quote! {
            async fn #ident(#self_param, #(#params),*) -> #return_ty {
                let vtable = #vtable_cell.get();
                ::uniffi::foreign_async_call::<_, #return_ty, crate::UniFfiTag>(
                    move |uniffi_future_callback, uniffi_future_callback_data, uniffi_foreign_future_dropped_callback| {
                        (vtable.#ident)(
                            self.handle,
                            #(#lower_exprs,)*
                            uniffi_future_callback,
                            uniffi_future_callback_data,
                            uniffi_foreign_future_dropped_callback
                        );
                }).await
            }
        })
    }
}

pub(super) fn metadata_items(
    self_ident: &Ident,
    items: &[ImplItem],
    module_path: &str,
    docstring: String,
) -> syn::Result<Vec<TokenStream>> {
    let trait_name = ident_to_string(self_ident);
    let callback_interface_items = create_metadata_items(
        "callback_interface",
        &trait_name,
        quote! {
            ::uniffi::MetadataBuffer::from_code(::uniffi::metadata::codes::CALLBACK_INTERFACE)
                .concat_str(#module_path)
                .concat_str(#trait_name)
                .concat_long_str(#docstring)
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
