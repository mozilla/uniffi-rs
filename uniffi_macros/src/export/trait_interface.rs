/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, quote_spanned};
use syn::ItemTrait;

use crate::{
    export::{
        attributes::ExportAttributeArguments, callback_interface, gen_method_scaffolding,
        item::ImplItem,
    },
    object::interface_meta_static_var,
    util::{derive_ffi_traits, ident_to_string, tagged_impl_header},
};

pub(super) fn gen_trait_scaffolding(
    mod_path: &str,
    args: ExportAttributeArguments,
    self_ident: Ident,
    items: Vec<ImplItem>,
    udl_mode: bool,
) -> syn::Result<TokenStream> {
    if let Some(rt) = args.async_runtime {
        return Err(syn::Error::new_spanned(rt, "not supported for traits"));
    }
    let trait_name = ident_to_string(&self_ident);
    let trait_impl = callback_interface::trait_impl(mod_path, &self_ident, &items, true)
        .unwrap_or_else(|e| e.into_compile_error());
    let inc_ref_fn_ident = Ident::new(
        &uniffi_meta::inc_ref_fn_symbol_name(mod_path, &trait_name),
        Span::call_site(),
    );
    let free_fn_ident = Ident::new(
        &uniffi_meta::free_fn_symbol_name(mod_path, &trait_name),
        Span::call_site(),
    );

    let helper_ffi_fn_tokens = quote! {
        #[doc(hidden)]
        #[no_mangle]
        pub extern "C" fn #inc_ref_fn_ident(
            handle: ::uniffi::Handle,
            call_status: &mut ::uniffi::RustCallStatus
        ) {
            uniffi::rust_call(call_status, || {
                <dyn #self_ident as ::uniffi::SlabAlloc<crate::UniFfiTag>>::inc_ref(handle);
                Ok(())
            });
        }

        #[doc(hidden)]
        #[no_mangle]
        pub extern "C" fn #free_fn_ident(
            handle: ::uniffi::Handle,
            call_status: &mut ::uniffi::RustCallStatus
        ) {
            uniffi::rust_call(call_status, || {
                <dyn #self_ident as ::uniffi::SlabAlloc<crate::UniFfiTag>>::remove(handle);
                Ok(())
            });
        }
    };

    let impl_tokens: TokenStream = items
        .into_iter()
        .map(|item| match item {
            ImplItem::Method(sig) => {
                if sig.is_async {
                    return Err(syn::Error::new(
                        sig.span,
                        "async trait methods are not supported",
                    ));
                }
                gen_method_scaffolding(sig, &args, udl_mode)
            }
            _ => unreachable!("traits have no constructors"),
        })
        .collect::<syn::Result<_>>()?;

    let meta_static_var = (!udl_mode).then(|| {
        interface_meta_static_var(&self_ident, true, mod_path)
            .unwrap_or_else(syn::Error::into_compile_error)
    });
    let ffi_converter_tokens = ffi_converter(mod_path, &self_ident, false);

    Ok(quote_spanned! { self_ident.span() =>
        #meta_static_var
        #helper_ffi_fn_tokens
        #trait_impl
        #impl_tokens
        #ffi_converter_tokens
    })
}

pub(crate) fn ffi_converter(mod_path: &str, trait_ident: &Ident, udl_mode: bool) -> TokenStream {
    let impl_spec = tagged_impl_header("FfiConverterArc", &quote! { dyn #trait_ident }, udl_mode);
    let lift_ref_impl_spec = tagged_impl_header("LiftRef", &quote! { dyn #trait_ident }, udl_mode);
    let derive_ffi_traits =
        derive_ffi_traits(&quote! { dyn #trait_ident }, udl_mode, &["SlabAlloc"]);
    let trait_name = ident_to_string(trait_ident);
    let trait_impl_ident = callback_interface::trait_impl_ident(&trait_name);

    quote! {
        // All traits must be `Sync + Send`. The generated scaffolding will fail to compile
        // if they are not, but unfortunately it fails with an unactionably obscure error message.
        // By asserting the requirement explicitly, we help Rust produce a more scrutable error message
        // and thus help the user debug why the requirement isn't being met.
        uniffi::deps::static_assertions::assert_impl_all!(dyn #trait_ident: Sync, Send);

        #derive_ffi_traits

        unsafe #impl_spec {
            type FfiType = ::uniffi::Handle;

            fn lower(obj: ::std::sync::Arc<Self>) -> ::uniffi::Handle {
                // If obj wraps a foreign implementation, then `uniffi_foreign_handle` will return
                // the handle here and we can use that rather than wrapping it again with Rust.
                let handle = match obj.uniffi_foreign_handle() {
                    Some(handle) => handle,
                    None => <dyn #trait_ident as ::uniffi::SlabAlloc<crate::UniFfiTag>>::insert(obj),
                };
                handle

            }

            fn try_lift(handle: ::uniffi::Handle) -> ::uniffi::deps::anyhow::Result<::std::sync::Arc<Self>> {
                Ok(if handle.is_foreign() {
                    // For foreign handles, construct a struct that implements the trait by calling
                    // the handle
                    ::std::sync::Arc::new(<#trait_impl_ident>::new(handle))
                } else {
                    // For Rust handles, remove the `Arc<>` from our slab.
                    <dyn #trait_ident as ::uniffi::SlabAlloc<crate::UniFfiTag>>::remove(handle)
                })
            }

            fn write(obj: ::std::sync::Arc<Self>, buf: &mut Vec<u8>) {
                ::uniffi::deps::bytes::BufMut::put_i64(
                    buf,
                    <Self as ::uniffi::FfiConverterArc<crate::UniFfiTag>>::lower(obj).as_raw(),
                );
            }

            fn try_read(buf: &mut &[u8]) -> ::uniffi::Result<::std::sync::Arc<Self>> {
                ::uniffi::check_remaining(buf, 8)?;
                <Self as ::uniffi::FfiConverterArc<crate::UniFfiTag>>::try_lift(
                    ::uniffi::Handle::from_raw(::uniffi::deps::bytes::Buf::get_i64(buf))
                )
            }

            const TYPE_ID_META: ::uniffi::MetadataBuffer = ::uniffi::MetadataBuffer::from_code(::uniffi::metadata::codes::TYPE_INTERFACE)
                .concat_str(#mod_path)
                .concat_str(#trait_name)
                .concat_bool(true);
        }

        unsafe #lift_ref_impl_spec {
            type LiftType = ::std::sync::Arc<dyn #trait_ident>;
        }
    }
}

pub fn alter_trait(item: &ItemTrait) -> TokenStream {
    let ItemTrait {
        attrs,
        vis,
        unsafety,
        auto_token,
        trait_token,
        ident,
        generics,
        colon_token,
        supertraits,
        items,
        ..
    } = item;

    quote! {
        #(#attrs)*
        #vis #unsafety #auto_token #trait_token #ident #generics #colon_token #supertraits {
            #(#items)*

            #[doc(hidden)]
            fn uniffi_foreign_handle(&self) -> Option<::uniffi::Handle> {
                None
            }
        }
    }
}
