use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{DeriveInput, Path};
use uniffi_meta::free_fn_symbol_name;

use crate::util::{
    create_metadata_items, ident_to_string, mod_path, tagged_impl_header, ArgumentNotAllowedHere,
    AttributeSliceExt, CommonAttr,
};

pub fn expand_object(input: DeriveInput, module_path: String) -> TokenStream {
    let ident = &input.ident;
    let attr_error = input
        .attrs
        .parse_uniffi_attr_args::<ArgumentNotAllowedHere>()
        .err()
        .map(syn::Error::into_compile_error);
    let name = ident_to_string(ident);
    let free_fn_ident = Ident::new(&free_fn_symbol_name(&module_path, &name), Span::call_site());
    let meta_static_var = interface_meta_static_var(ident, false, &module_path)
        .unwrap_or_else(syn::Error::into_compile_error);
    let interface_impl = interface_impl(ident, None);

    quote! {
        #[doc(hidden)]
        #[no_mangle]
        pub extern "C" fn #free_fn_ident(
            ptr: *const ::std::ffi::c_void,
            call_status: &mut ::uniffi::RustCallStatus
        ) {
            uniffi::rust_call(call_status, || {
                assert!(!ptr.is_null());
                let ptr = ptr.cast::<#ident>();
                unsafe {
                    ::std::sync::Arc::decrement_strong_count(ptr);
                }
                Ok(())
            });
        }

        #attr_error
        #interface_impl
        #meta_static_var
    }
}

pub(crate) fn expand_interface_support(attr: CommonAttr, input: DeriveInput) -> TokenStream {
    interface_impl(&input.ident, attr.tag.as_ref())
}

pub(crate) fn interface_impl(ident: &Ident, tag: Option<&Path>) -> TokenStream {
    let name = ident_to_string(ident);
    let impl_spec = tagged_impl_header("FfiConverterArc", ident, tag);
    let mod_path = match mod_path() {
        Ok(p) => p,
        Err(e) => return e.into_compile_error(),
    };

    quote! {
        // All Object structs must be `Sync + Send`. The generated scaffolding will fail to compile
        // if they are not, but unfortunately it fails with an unactionably obscure error message.
        // By asserting the requirement explicitly, we help Rust produce a more scrutable error message
        // and thus help the user debug why the requirement isn't being met.
        uniffi::deps::static_assertions::assert_impl_all!(#ident: Sync, Send);

        #[doc(hidden)]
        #[automatically_derived]
        /// Support for passing reference-counted shared objects via the FFI.
        ///
        /// To avoid dealing with complex lifetime semantics over the FFI, any data passed
        /// by reference must be encapsulated in an `Arc`, and must be safe to share
        /// across threads.
        unsafe #impl_spec {
            // Don't use a pointer to <T> as that requires a `pub <T>`
            type FfiType = *const ::std::os::raw::c_void;
            type ReturnType = *const ::std::os::raw::c_void;
            type FutureCallback = ::uniffi::FutureCallback<Self::ReturnType>;

            /// When lowering, we have an owned `Arc` and we transfer that ownership
            /// to the foreign-language code, "leaking" it out of Rust's ownership system
            /// as a raw pointer. This works safely because we have unique ownership of `self`.
            /// The foreign-language code is responsible for freeing this by calling the
            /// `ffi_object_free` FFI function provided by the corresponding UniFFI type.
            ///
            /// Safety: when freeing the resulting pointer, the foreign-language code must
            /// call the destructor function specific to the type `T`. Calling the destructor
            /// function for other types may lead to undefined behaviour.
            fn lower(obj: ::std::sync::Arc<Self>) -> Self::FfiType {
                ::std::sync::Arc::into_raw(obj) as Self::FfiType
            }

            /// When lifting, we receive a "borrow" of the `Arc` that is owned by
            /// the foreign-language code, and make a clone of it for our own use.
            ///
            /// Safety: the provided value must be a pointer previously obtained by calling
            /// the `lower()` or `write()` method of this impl.
            fn try_lift(v: Self::FfiType) -> ::uniffi::Result<::std::sync::Arc<Self>> {
                let v = v as *const #ident;
                // We musn't drop the `Arc` that is owned by the foreign-language code.
                let foreign_arc = ::std::mem::ManuallyDrop::new(unsafe { ::std::sync::Arc::<Self>::from_raw(v) });
                // Take a clone for our own use.
                Ok(::std::sync::Arc::clone(&*foreign_arc))
            }

            /// When writing as a field of a complex structure, make a clone and transfer ownership
            /// of it to the foreign-language code by writing its pointer into the buffer.
            /// The foreign-language code is responsible for freeing this by calling the
            /// `ffi_object_free` FFI function provided by the corresponding UniFFI type.
            ///
            /// Safety: when freeing the resulting pointer, the foreign-language code must
            /// call the destructor function specific to the type `T`. Calling the destructor
            /// function for other types may lead to undefined behaviour.
            fn write(obj: ::std::sync::Arc<Self>, buf: &mut Vec<u8>) {
                ::uniffi::deps::static_assertions::const_assert!(::std::mem::size_of::<*const ::std::ffi::c_void>() <= 8);
                ::uniffi::deps::bytes::BufMut::put_u64(buf, <Self as ::uniffi::FfiConverterArc<crate::UniFfiTag>>::lower(obj) as u64);
            }

            /// When reading as a field of a complex structure, we receive a "borrow" of the `Arc`
            /// that is owned by the foreign-language code, and make a clone for our own use.
            ///
            /// Safety: the buffer must contain a pointer previously obtained by calling
            /// the `lower()` or `write()` method of this impl.
            fn try_read(buf: &mut &[u8]) -> ::uniffi::Result<::std::sync::Arc<Self>> {
                ::uniffi::deps::static_assertions::const_assert!(::std::mem::size_of::<*const ::std::ffi::c_void>() <= 8);
                ::uniffi::check_remaining(buf, 8)?;
                <Self as ::uniffi::FfiConverterArc<crate::UniFfiTag>>::try_lift(::uniffi::deps::bytes::Buf::get_u64(buf) as Self::FfiType)
            }

            fn lower_return(v: ::std::sync::Arc<Self>) -> ::std::result::Result<Self::FfiType, ::uniffi::RustBuffer> {
                Ok(<Self as ::uniffi::FfiConverterArc<crate::UniFfiTag>>::lower(v))
            }

            fn invoke_future_callback(
                callback: Self::FutureCallback,
                callback_data: *const (),
                return_value: Self::ReturnType,
                call_status: ::uniffi::RustCallStatus,
            ) {
                callback(callback_data, return_value, call_status);
            }

            const TYPE_ID_META: ::uniffi::MetadataBuffer = ::uniffi::MetadataBuffer::from_code(::uniffi::metadata::codes::TYPE_INTERFACE)
                .concat_str(#mod_path)
                .concat_str(#name)
                .concat_bool(false);
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
