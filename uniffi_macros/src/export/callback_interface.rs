/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::{
    export::ImplItem,
    ffiops,
    fnsig::{FnKind, FnSignature, ReceiverArg},
    util::{
        async_trait_annotation, create_metadata_items, derive_ffi_traits, ident_to_string,
        tagged_impl_header, wasm_single_threaded_annotation,
    },
};
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, ToTokens};
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
    for_trait_interface: bool,
) -> syn::Result<TokenStream> {
    let trait_name = ident_to_string(trait_ident);
    let switch = trait_switch_ident(&trait_name);
    let methods = items
        .iter()
        .map(CallbackMethod::try_from)
        .collect::<syn::Result<Vec<_>>>()?;

    let mut impls = TokenStream::default();
    impls.extend(trait_impl_legacy_ffi(
        mod_path,
        trait_ident,
        &trait_name,
        &methods,
        for_trait_interface,
    )?);

    #[cfg(feature = "pointer-ffi")]
    impls.extend(trait_impl_pointer_ffi(
        mod_path,
        trait_ident,
        &trait_name,
        &methods,
        for_trait_interface,
    )?);

    Ok(quote! {
        static #switch: ::std::sync::atomic::AtomicUsize = ::std::sync::atomic::AtomicUsize::new(0);

        #impls
    })
}

fn trait_impl_legacy_ffi(
    mod_path: &str,
    trait_ident: &Ident,
    trait_name: &str,
    methods: &[CallbackMethod<'_>],
    for_trait_interface: bool,
) -> syn::Result<TokenStream> {
    let trait_impl_ident = Ident::new(
        &format!("UniFFICallbackHandler{trait_name}"),
        Span::call_site(),
    );
    let vtable_type = format_ident!("UniFfiTraitVtable{trait_name}");
    let vtable_cell = format_ident!("UNIFFI_TRAIT_CELL_{}", trait_name.to_uppercase());
    let init_ident = Ident::new(
        &uniffi_meta::init_callback_vtable_fn_symbol_name(mod_path, &trait_name),
        Span::call_site(),
    );

    let uniffi_foreign_handle_method = for_trait_interface.then(|| {
        quote! {
            fn uniffi_foreign_handle(&self) -> ::std::option::Option<::uniffi::Handle> {
                let vtable = #vtable_cell.get();
                ::std::option::Option::Some(::uniffi::Handle::from_raw_unchecked((vtable.uniffi_clone)(self.handle)))
            }
        }
    });

    let vtable_fields = methods.iter().map(|meth| {
        let ident = &meth.ident;
        let param_names = &meth.scaffolding_param_names;
        let param_types = &meth.scaffolding_param_types;
        let lift_return_type = &meth.lift_return_type;
        if !meth.is_async {
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
        .map(|meth| gen_method_impl(meth, &vtable_cell))
        .collect::<syn::Result<Vec<_>>>()?;
    let has_async_method = methods.iter().any(|m| m.is_async);

    // Conditionally apply the async_trait attribute with or without ?Send based on the target
    let impl_attributes = has_async_method.then(async_trait_annotation);

    let single_threaded_annotation = wasm_single_threaded_annotation();

    Ok(quote! {
        #[allow(missing_docs)]
        pub struct #vtable_type {
            pub uniffi_free: extern "C" fn(handle: u64),
            pub uniffi_clone: extern "C" fn(handle: u64) -> u64,
            #(#vtable_fields)*
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
            #uniffi_foreign_handle_method
        }

        impl ::std::ops::Drop for #trait_impl_ident {
            fn drop(&mut self) {
                let vtable = #vtable_cell.get();
                (vtable.uniffi_free)(self.handle);
            }
        }
    })
}

/// Expression to construct a `Box<dyn trait>` or `Arc<dyn Trait>`
///
/// If the `pointer-ffi` feature is enabled, then we generate two structs that implement our
/// callback traits.  One uses the legacy FFI and one uses the pointer FFI.  We may add more
/// options in the future.  This function generates an expression to pick the correct one and
/// construct it from a handle.
///
/// `container` must be either `::std::box::Box` or `::std::sync::Arc`.
pub fn construct_dyn_trait(
    trait_name: &str,
    container: impl ToTokens,
    handle: impl ToTokens,
) -> TokenStream {
    let switch = trait_switch_ident(trait_name);
    let mut cases = TokenStream::default();
    let mut add_case = |value: usize, impl_name: String| {
        let impl_ident = Ident::new(&impl_name, Span::call_site());
        cases.extend(quote! { #value => #container::new(#impl_ident::new(#handle)), })
    };
    add_case(0, format!("UniFFICallbackHandler{trait_name}"));
    #[cfg(feature = "pointer-ffi")]
    add_case(1, format!("UniffiPointerCallbackHandler{trait_name}"));

    quote! {
        match #switch.load(::std::sync::atomic::Ordering::Relaxed) {
            #cases
            n => ::core::panic!("UniFFI: Invalid callback impl value for {} ({})", #trait_name, n),
        }
    }
}

/// Identifier for the AtomicUsize used to switch between trait implementations
fn trait_switch_ident(trait_name: &str) -> Ident {
    Ident::new(
        &format!("UNIFFI_TRAIT_SWITCH_{trait_name}"),
        Span::call_site(),
    )
}

pub fn ffi_converter_callback_interface_impl(trait_ident: &Ident) -> TokenStream {
    // TODO: support remote callback interfaces
    let remote = false;
    let trait_name = ident_to_string(trait_ident);
    let dyn_trait = quote! { dyn #trait_ident };
    let box_dyn_trait = quote! { ::std::boxed::Box<#dyn_trait> };
    let lift_impl_spec = tagged_impl_header("Lift", &box_dyn_trait, remote);
    let type_id_impl_specs = [
        tagged_impl_header("TypeId", &box_dyn_trait, remote),
        tagged_impl_header("TypeId", &dyn_trait, remote),
    ]
    .into_iter();
    let derive_ffi_traits = derive_ffi_traits(&box_dyn_trait, remote, &["LiftRef", "LiftReturn"]);
    let try_lift_self = ffiops::try_lift(quote! { Self });
    let construct_dyn_trait =
        construct_dyn_trait(&trait_name, quote! { ::std::boxed::Box }, quote! { v });

    quote! {
        #[doc(hidden)]
        #[automatically_derived]
        unsafe #lift_impl_spec {
            type FfiType = u64;

            fn try_lift(v: Self::FfiType) -> ::uniffi::deps::anyhow::Result<Self> {
                ::std::result::Result::Ok(#construct_dyn_trait)
            }

            fn try_read(buf: &mut &[u8]) -> ::uniffi::deps::anyhow::Result<Self> {
                use ::uniffi::deps::bytes::Buf;
                ::uniffi::check_remaining(buf, 8)?;
                #try_lift_self(buf.get_u64())
            }
        }

        #(
            #[doc(hidden)]
            #[automatically_derived]
            #type_id_impl_specs {
                const TYPE_ID_META: ::uniffi::MetadataBuffer = ::uniffi::MetadataBuffer::from_code(
                    ::uniffi::metadata::codes::TYPE_CALLBACK_INTERFACE,
                )
                .concat_str(module_path!())
                .concat_str(#trait_name);
            }
        )*

        #derive_ffi_traits
    }
}

/// Info we need to handle a callback interface method
///
/// This is used by both `trait_impl_legacy_ffi` and `trait_impl_pointer_ffi`
struct CallbackMethod<'a> {
    ident: &'a Ident,
    is_async: bool,
    return_ty: &'a TokenStream,
    kind: &'a FnKind,
    receiver: &'a Option<ReceiverArg>,
    name: &'a str,
    span: &'a Span,
    self_param: TokenStream,
    params: Vec<TokenStream>,
    scaffolding_param_names: Vec<TokenStream>,
    scaffolding_param_types: Vec<TokenStream>,
    lower_exprs: Vec<TokenStream>,
    lift_return_type: TokenStream,
    lift_foreign_return: TokenStream,
}

impl<'a> TryFrom<&'a ImplItem> for CallbackMethod<'a> {
    type Error = syn::Error;

    fn try_from(item: &'a ImplItem) -> syn::Result<Self> {
        let sig = match item {
            ImplItem::Constructor(sig) => {
                return Err(syn::Error::new(
                    sig.span,
                    "Constructors not allowed in trait interfaces",
                ));
            }
            ImplItem::Method(sig) => sig,
        };
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

        let lower_exprs = sig
            .args
            .iter()
            .map(|a| {
                let lower = ffiops::lower(&a.ty);
                let ident = &a.ident;
                quote! { #lower(#ident) }
            })
            .collect();

        let lift_return_type = ffiops::lift_return_type(&sig.return_ty);
        let lift_foreign_return = ffiops::lift_foreign_return(&sig.return_ty);

        Ok(Self {
            ident,
            is_async: *is_async,
            return_ty,
            kind,
            receiver,
            name,
            span,
            self_param,
            params: sig.params().collect(),
            scaffolding_param_names: sig.scaffolding_param_names().collect(),
            scaffolding_param_types: sig.scaffolding_param_types().collect(),
            lower_exprs,
            lift_return_type,
            lift_foreign_return,
        })
    }
}

/// Generate a single method for [trait_impl].  This implements a trait method by invoking a
/// foreign-supplied callback.
fn gen_method_impl(meth: &CallbackMethod<'_>, vtable_cell: &Ident) -> syn::Result<TokenStream> {
    let CallbackMethod {
        ident,
        is_async,
        return_ty,
        self_param,
        params,
        lower_exprs,
        lift_return_type,
        lift_foreign_return,
        ..
    } = meth;

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

#[cfg(feature = "pointer-ffi")]
fn trait_impl_pointer_ffi(
    mod_path: &str,
    trait_ident: &Ident,
    trait_name: &str,
    methods: &[CallbackMethod<'_>],
    for_trait_interface: bool,
) -> syn::Result<TokenStream> {
    let trait_impl_ident = Ident::new(
        &format!("UniffiPointerCallbackHandler{trait_name}"),
        Span::call_site(),
    );
    let switch = trait_switch_ident(trait_name);
    let vtable_type = format_ident!("UniffiPointerTraitVtable{trait_name}");
    let vtable_once = format_ident!("UNIFFI_POINTER_TRAIT_ONCE_{}", trait_name.to_uppercase());
    let init_ident = Ident::new(
        &uniffi_meta::pointer_ffi_symbol_name(&uniffi_meta::init_callback_vtable_fn_symbol_name(
            mod_path, trait_name,
        )),
        Span::call_site(),
    );

    let uniffi_foreign_handle_method = for_trait_interface.then(|| {
        quote! {
            fn uniffi_foreign_handle(&self) -> ::std::option::Option<::uniffi::Handle> {
                unsafe {
                    let vtable = #vtable_once.get().unwrap_or_else(|| ::core::panic!("UniFFI: vtable not initialized for {}", #trait_name));
                    let mut uniffi_ffi_buf = [0_u8; 8];
                    let mut uniffi_args_buf = uniffi_ffi_buf.as_mut_slice();
                    // We're passing the handle by reference, so we can use `clone_for_ref` to
                    // avoid an arc clone
                    <::uniffi::Handle as ::uniffi::FfiSerialize>::write(&mut uniffi_args_buf, self.handle.clone_for_ref());
                    (vtable.uniffi_clone)(uniffi_ffi_buf.as_mut_ptr());

                    let mut uniffi_return_buf = uniffi_ffi_buf.as_slice();
                    let uniffi_handle = <::uniffi::Handle as ::uniffi::FfiSerialize>::read(&mut uniffi_return_buf);

                    ::std::option::Option::Some(uniffi_handle)
                }
            }
        }
    });

    // Total number of methods for the vtable, one for each callback method plus free/clone
    let vtable_total_method_count = methods.len() + 2;

    let method_names: Vec<&Ident> = methods.iter().map(|meth| meth.ident).collect();

    let trait_impl_methods = methods
        .iter()
        .map(|sig| gen_method_impl_pointer_ffi(sig, trait_name, &vtable_once))
        .collect::<syn::Result<Vec<_>>>()?;
    let has_async_method = methods.iter().any(|m| m.is_async);

    // Conditionally apply the async_trait attribute with or without ?Send based on the target
    let impl_attributes = has_async_method.then(async_trait_annotation);

    let single_threaded_annotation = wasm_single_threaded_annotation();

    Ok(quote! {
        #[allow(missing_docs)]
        #[derive(::std::clone::Clone)]
        pub struct #vtable_type {
            pub uniffi_free: unsafe extern "C" fn(uniffi_buf: *mut u8),
            pub uniffi_clone: unsafe extern "C" fn(uniffi_buf: *mut u8),
            #(
                pub #method_names: unsafe extern "C" fn(uniffi_buf: *mut u8),
            )*
        }

        static #vtable_once: ::std::sync::OnceLock::<#vtable_type> = ::std::sync::OnceLock::new();

        #[allow(missing_docs)]
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn #init_ident(ffi_buffer: *mut u8) {
            let mut uniffi_args_buf = ::std::slice::from_raw_parts(
                ffi_buffer,
                <::uniffi::pointer_ffi::CallbackFn as ::uniffi::FfiSerialize>::SIZE * #vtable_total_method_count
            );
            let vtable = #vtable_type {
                uniffi_free: <::uniffi::pointer_ffi::CallbackFn as ::uniffi::FfiSerialize>::read(&mut uniffi_args_buf),
                uniffi_clone: <::uniffi::pointer_ffi::CallbackFn as ::uniffi::FfiSerialize>::read(&mut uniffi_args_buf),
                #(
                    #method_names: <::uniffi::pointer_ffi::CallbackFn as ::uniffi::FfiSerialize>::read(&mut uniffi_args_buf),
                )*
            };
            if #vtable_once.set(vtable).is_err() {
                ::core::panic!("UniFFI: vtable initialized multiple times ({})", #trait_name);
            }
            #switch.store(1, ::std::sync::atomic::Ordering::Relaxed);
        }

        #[derive(Debug)]
        struct #trait_impl_ident {
            handle: ::uniffi::Handle,
        }

        impl #trait_impl_ident {
            fn new(handle: u64) -> Self {
                Self { handle: ::uniffi::Handle::from_raw_unchecked(handle) }
            }
        }

        #single_threaded_annotation
        ::uniffi::deps::static_assertions::assert_impl_all!(#trait_impl_ident: ::core::marker::Send);

        #impl_attributes
        impl #trait_ident for #trait_impl_ident {
            #(#trait_impl_methods)*
            #uniffi_foreign_handle_method
        }

        impl ::std::ops::Drop for #trait_impl_ident {
            fn drop(&mut self) {
                unsafe {
                    let vtable = #vtable_once.get().unwrap_or_else(|| ::core::panic!("UniFFI: vtable not initialized for {}", #trait_name));
                    let mut uniffi_ffi_buf = [0_u8; 8];
                    let mut uniffi_args_buf = uniffi_ffi_buf.as_mut_slice();
                    // We're passing the handle by reference, so we can use `clone_for_ref` to
                    // avoid an arc clone
                    <::uniffi::Handle as ::uniffi::FfiSerialize>::write(&mut uniffi_args_buf, self.handle.clone_for_ref());
                    (vtable.uniffi_free)(uniffi_ffi_buf.as_mut_ptr());
                }
            }
        }
    })
}

#[cfg(feature = "pointer-ffi")]
/// Generate a single method for [trait_impl].  This implements a trait method by invoking a
/// foreign-supplied callback.
fn gen_method_impl_pointer_ffi(
    meth: &CallbackMethod<'_>,
    trait_name: &str,
    vtable_once: &Ident,
) -> syn::Result<TokenStream> {
    let CallbackMethod {
        ident,
        is_async,
        return_ty,
        self_param,
        params,
        lower_exprs,
        scaffolding_param_types,
        lift_return_type,
        lift_foreign_return,
        name,
        ..
    } = meth;

    if !is_async {
        Ok(quote! {
            fn #ident(#self_param, #(#params),*) -> #return_ty {
                // Safety: we follow the pointer FFI when reading/writing from the buffer
                unsafe {
                    ::uniffi::trace!("[pointer-ffi] Calling callback interface method {}::{} (handle: {:?})", #trait_name, #name, self.handle);
                    let vtable = #vtable_once.get().unwrap_or_else(|| ::core::panic!("UniFFI: vtable not initialized for {}", #trait_name));

                    let mut uniffi_ffi_buf = [
                        0_u8;
                        ::uniffi::ffi_buffer_size!(
                            (::uniffi::Handle #(, #scaffolding_param_types)*),
                            (::uniffi::RustCallStatus, #lift_return_type)
                        )
                    ];
                    let mut uniffi_args_buf = uniffi_ffi_buf.as_mut_slice();
                    // We're passing the handle by reference, so we can use `clone_for_ref` to
                    // avoid an arc clone
                    <::uniffi::Handle as ::uniffi::FfiSerialize>::write(&mut uniffi_args_buf, self.handle.clone_for_ref());
                    #(
                        <#scaffolding_param_types as ::uniffi::FfiSerialize>::write(&mut uniffi_args_buf, #lower_exprs);
                    )*
                    ::uniffi::trace!("calling foreign impl {}::{}", #trait_name, #name);

                    (vtable.#ident)(uniffi_ffi_buf.as_mut_ptr());

                    ::uniffi::trace!("call complete {}::{}", #trait_name, #name);

                    let mut uniffi_return_buf = uniffi_ffi_buf.as_slice();
                    let uniffi_call_status = <::uniffi::RustCallStatus as ::uniffi::FfiSerialize>::read(&mut uniffi_return_buf);
                    let uniffi_return_value = <#lift_return_type as ::uniffi::FfiSerialize>::read(&mut uniffi_return_buf);
                    ::uniffi::trace!("read result {}::{} ({uniffi_call_status:?} / {uniffi_return_value:?})", #trait_name, #name);
                    #lift_foreign_return(uniffi_return_value, uniffi_call_status)
                }
            }
        })
    } else {
        Ok(quote! {
            async fn #ident(#self_param, #(#params),*) -> #return_ty {
                todo!()
            }
        })
    }
}

pub(super) fn metadata_items(
    self_ident: &Ident,
    items: &[ImplItem],
    docstring: String,
) -> syn::Result<Vec<TokenStream>> {
    let trait_name = ident_to_string(self_ident);
    let callback_interface_items = create_metadata_items(
        "callback_interface",
        &trait_name,
        quote! {
            ::uniffi::MetadataBuffer::from_code(::uniffi::metadata::codes::CALLBACK_INTERFACE)
                .concat_str(module_path!())
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
