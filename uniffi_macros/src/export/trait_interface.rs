/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, quote_spanned};

use uniffi_meta::ObjectImpl;

use crate::{
    export::{
        attributes::ExportTraitArgs, callback_interface, gen_method_scaffolding, item::ImplItem,
    },
    ffiops,
    object::interface_meta_static_var,
    util::{ident_to_string, tagged_impl_header, wasm_single_threaded_annotation},
};

pub(super) fn gen_trait_scaffolding(
    mod_path: &str,
    args: ExportTraitArgs,
    self_ident: Ident,
    items: Vec<ImplItem>,
    udl_mode: bool,
    with_foreign: bool,
    docstring: String,
) -> syn::Result<TokenStream> {
    if let Some(rt) = args.async_runtime {
        return Err(syn::Error::new_spanned(rt, "not supported for traits"));
    }
    let trait_name = ident_to_string(&self_ident);
    let trait_impl = with_foreign.then(|| {
        callback_interface::trait_impl(mod_path, &self_ident, &items)
            .unwrap_or_else(|e| e.into_compile_error())
    });

    let clone_fn_ident = Ident::new(
        &uniffi_meta::clone_fn_symbol_name(mod_path, &trait_name),
        Span::call_site(),
    );
    let free_fn_ident = Ident::new(
        &uniffi_meta::free_fn_symbol_name(mod_path, &trait_name),
        Span::call_site(),
    );

    let helper_fn_tokens = quote! {
        #[doc(hidden)]
        #[unsafe(no_mangle)]
        /// Clone a pointer to this object type
        ///
        /// Safety: Only pass pointers returned by a UniFFI call.  Do not pass pointers that were
        /// passed to the free function.
        pub unsafe extern "C" fn #clone_fn_ident(
            handle: ::uniffi::ffi::Handle,
            call_status: &mut ::uniffi::RustCallStatus
        ) -> ::uniffi::ffi::Handle {
            ::uniffi::deps::trace!("clonining trait: {} ({:x})", #trait_name, handle);
            ::uniffi::rust_call(call_status, || {
                unsafe {
                    ::std::result::Result::Ok(
                        handle.clone_arc_handle::<::std::sync::Arc<dyn #self_ident>>()
                    )
                }
            })
        }

        #[doc(hidden)]
        #[unsafe(no_mangle)]
        /// Free a pointer to this object type
        ///
        /// Safety: Only pass pointers returned by a UniFFI call.  Do not pass pointers that were
        /// passed to the free function.
        ///
        /// Note: clippy doesn't complain about this being unsafe, but it definitely is since it
        /// calls `Box::from_raw`.
        pub unsafe extern "C" fn #free_fn_ident(
            handle: ::uniffi::ffi::Handle,
            call_status: &mut ::uniffi::RustCallStatus
        ) {
            ::uniffi::deps::trace!("freeing trait: {} ({:x})", #trait_name, handle);
            ::uniffi::rust_call(call_status, || {
                ::std::mem::drop(unsafe {
                    handle.into_arc::<::std::sync::Arc<dyn #self_ident>>()
                });
                ::std::result::Result::Ok(())
            });
        }
    };

    let impl_tokens: TokenStream = items
        .into_iter()
        .map(|item| match item {
            ImplItem::Method(sig) => gen_method_scaffolding(sig, None, udl_mode),
            _ => unreachable!("traits have no constructors"),
        })
        .collect::<syn::Result<_>>()?;

    let meta_static_var = (!udl_mode).then(|| {
        let imp = if with_foreign {
            ObjectImpl::CallbackTrait
        } else {
            ObjectImpl::Trait
        };
        interface_meta_static_var(&self_ident, imp, mod_path, docstring.as_str())
            .unwrap_or_else(syn::Error::into_compile_error)
    });
    let ffi_converter_tokens = ffi_converter(mod_path, &self_ident, with_foreign);

    Ok(quote_spanned! { self_ident.span() =>
        #meta_static_var
        #helper_fn_tokens
        #trait_impl
        #impl_tokens
        #ffi_converter_tokens
    })
}

pub(crate) fn ffi_converter(
    mod_path: &str,
    trait_ident: &Ident,
    with_foreign: bool,
) -> TokenStream {
    // TODO: support defining remote trait interfaces
    let remote = false;
    let impl_spec = tagged_impl_header("FfiConverterArc", &quote! { dyn #trait_ident }, remote);
    let lift_ref_impl_spec = tagged_impl_header("LiftRef", &quote! { dyn #trait_ident }, remote);
    let trait_name = ident_to_string(trait_ident);
    let try_lift = if with_foreign {
        let trait_impl_ident = callback_interface::trait_impl_ident(&trait_name);
        quote! {
            fn try_lift(handle: Self::FfiType) -> ::uniffi::deps::anyhow::Result<::std::sync::Arc<Self>> {
                ::std::result::Result::Ok(::std::sync::Arc::new(<#trait_impl_ident>::new(handle.as_raw())))
            }
        }
    } else {
        quote! {
            fn try_lift(handle: Self::FfiType) -> ::uniffi::deps::anyhow::Result<::std::sync::Arc<Self>> {
                use ::std::clone::Clone;
                // Note: handle is for a double-wrapped arc
                // https://mozilla.github.io/uniffi-rs/latest/internals/object_references.html
                let obj: ::std::sync::Arc<::std::sync::Arc<dyn #trait_ident>> = unsafe {
                    handle.into_arc()
                };
                ::std::result::Result::Ok((*obj).clone())
            }
        }
    };
    let metadata_code = if with_foreign {
        quote! { ::uniffi::metadata::codes::TYPE_CALLBACK_TRAIT_INTERFACE }
    } else {
        quote! { ::uniffi::metadata::codes::TYPE_TRAIT_INTERFACE }
    };
    let lower_self = ffiops::lower(quote! { ::std::sync::Arc<Self> });
    let try_lift_self = ffiops::try_lift(quote! { ::std::sync::Arc<Self> });
    let single_threaded_annotation = wasm_single_threaded_annotation();

    quote! {
        // All traits must be `Sync + Send`. The generated scaffolding will fail to compile
        // if they are not, but unfortunately it fails with an unactionably obscure error message.
        // By asserting the requirement explicitly, we help Rust produce a more scrutable error message
        // and thus help the user debug why the requirement isn't being met.
        #single_threaded_annotation
        ::uniffi::deps::static_assertions::assert_impl_all!(
            dyn #trait_ident: ::core::marker::Sync, ::core::marker::Send
        );

        // We're going to be casting raw pointers to `u64` values to pass them across the FFI.
        // Ensure that we're not on some 128-bit machine where this would overflow.
        ::uniffi::deps::static_assertions::const_assert!(::std::mem::size_of::<*const ()>() <= 8);

        unsafe #impl_spec {
            type FfiType = ::uniffi::ffi::Handle;


            fn lower(obj: ::std::sync::Arc<Self>) -> Self::FfiType {
                use ::std::sync::Arc;
                // Wrap `obj` in a second arc
                // https://mozilla.github.io/uniffi-rs/latest/internals/object_references.html
                let obj: Arc<Arc<dyn #trait_ident>> = Arc::new(obj);
                ::uniffi::ffi::Handle::from_arc(obj)
            }

            #try_lift

            fn write(obj: ::std::sync::Arc<Self>, buf: &mut ::std::vec::Vec<u8>) {
                ::uniffi::deps::bytes::BufMut::put_u64(
                    buf,
                    #lower_self(obj).as_raw()
                );
            }

            fn try_read(buf: &mut &[u8]) -> ::uniffi::Result<::std::sync::Arc<Self>> {
                ::uniffi::check_remaining(buf, 8)?;
                #try_lift_self(::uniffi::ffi::Handle::from_raw_unchecked(::uniffi::deps::bytes::Buf::get_u64(buf)))
            }

            const TYPE_ID_META: ::uniffi::MetadataBuffer = ::uniffi::MetadataBuffer::from_code(#metadata_code)
                .concat_str(#mod_path)
                .concat_str(#trait_name);
        }

        unsafe #lift_ref_impl_spec {
            type LiftType = ::std::sync::Arc<dyn #trait_ident>;
        }
    }
}
