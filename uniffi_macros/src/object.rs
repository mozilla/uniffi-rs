use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::DeriveInput;

use crate::util::{create_metadata_items, ident_to_string, mod_path, tagged_impl_header};

pub fn expand_object(input: DeriveInput, udl_mode: bool) -> syn::Result<TokenStream> {
    let module_path = mod_path()?;
    let ident = &input.ident;
    let name = ident_to_string(ident);
    let clone_fn_ident = Ident::new(
        &uniffi_meta::clone_fn_symbol_name(&module_path, &name),
        Span::call_site(),
    );
    let free_fn_ident = Ident::new(
        &uniffi_meta::free_fn_symbol_name(&module_path, &name),
        Span::call_site(),
    );
    let meta_static_var = (!udl_mode).then(|| {
        interface_meta_static_var(ident, false, &module_path)
            .unwrap_or_else(syn::Error::into_compile_error)
    });
    let interface_impl = interface_impl(ident, udl_mode);

    Ok(quote! {
        #[doc(hidden)]
        #[no_mangle]
        pub extern "C" fn #clone_fn_ident(
            handle: ::uniffi::Handle,
            call_status: &mut ::uniffi::RustCallStatus
        ) -> ::uniffi::Handle {
            uniffi::rust_call(call_status, || {
                Ok(<#ident as ::uniffi::HandleAlloc<crate::UniFfiTag>>::clone_handle(handle))
            })
        }

        #[doc(hidden)]
        #[no_mangle]
        pub extern "C" fn #free_fn_ident(
            handle: ::uniffi::Handle,
            call_status: &mut ::uniffi::RustCallStatus
        ) {
            uniffi::rust_call(call_status, || {
                <#ident as ::uniffi::HandleAlloc<crate::UniFfiTag>>::consume_handle(handle);
                Ok(())
            });
        }

        #interface_impl
        #meta_static_var
    })
}

pub(crate) fn interface_impl(ident: &Ident, udl_mode: bool) -> TokenStream {
    let name = ident_to_string(ident);
    let impl_spec = tagged_impl_header("FfiConverterArc", ident, udl_mode);
    let lift_ref_impl_spec = tagged_impl_header("LiftRef", ident, udl_mode);
    let mod_path = match mod_path() {
        Ok(p) => p,
        Err(e) => return e.into_compile_error(),
    };

    quote! {
        // All Object structs must be `Sync + Send`. The generated scaffolding will fail to compile
        // if they are not, but unfortunately it fails with an unactionably obscure error message.
        // By asserting the requirement explicitly, we help Rust produce a more scrutable error message
        // and thus help the user debug why the requirement isn't being met.
        uniffi::deps::static_assertions::assert_impl_all!(#ident: ::core::marker::Sync, ::core::marker::Send);

        #[doc(hidden)]
        #[automatically_derived]
        /// Support for passing reference-counted shared objects via the FFI.
        ///
        /// Objects are `Arc<>` values in Rust and Handle values on the FFI.
        /// The `HandleAlloc` trait doc string has usage guidelines for handles.
        unsafe #impl_spec {
            type FfiType = ::uniffi::Handle;

            fn lower(obj: ::std::sync::Arc<Self>) -> Self::FfiType {
                <#ident as ::uniffi::HandleAlloc<crate::UniFfiTag>>::new_handle(obj)
            }

            fn try_lift(handle: Self::FfiType) -> ::uniffi::Result<::std::sync::Arc<Self>> {
                Ok(<#ident as ::uniffi::HandleAlloc<crate::UniFfiTag>>::consume_handle(handle))
            }

            fn write(obj: ::std::sync::Arc<Self>, buf: &mut Vec<u8>) {
                ::uniffi::deps::bytes::BufMut::put_u64(buf, <Self as ::uniffi::FfiConverterArc<crate::UniFfiTag>>::lower(obj).as_raw())
            }

            fn try_read(buf: &mut &[u8]) -> ::uniffi::Result<::std::sync::Arc<Self>> {
                ::uniffi::check_remaining(buf, 8)?;
                <Self as ::uniffi::FfiConverterArc<crate::UniFfiTag>>::try_lift(
                    ::uniffi::Handle::from_raw_unchecked(::uniffi::deps::bytes::Buf::get_u64(buf))
                )
            }

            const TYPE_ID_META: ::uniffi::MetadataBuffer = ::uniffi::MetadataBuffer::from_code(::uniffi::metadata::codes::TYPE_INTERFACE)
                .concat_str(#mod_path)
                .concat_str(#name)
                .concat_bool(false);
        }

        unsafe #lift_ref_impl_spec {
            type LiftType = ::std::sync::Arc<Self>;
        }
    }
}

pub(crate) fn interface_meta_static_var(
    ident: &Ident,
    is_trait: bool,
    module_path: &str,
) -> syn::Result<TokenStream> {
    let name = ident_to_string(ident);
    Ok(create_metadata_items(
        "interface",
        &name,
        quote! {
                ::uniffi::MetadataBuffer::from_code(::uniffi::metadata::codes::INTERFACE)
                    .concat_str(#module_path)
                    .concat_str(#name)
                    .concat_bool(#is_trait)
        },
        None,
    ))
}
