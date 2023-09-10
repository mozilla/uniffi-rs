/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Result;

use crate::util::mod_path;
use uniffi_meta::UNIFFI_CONTRACT_VERSION;

pub fn setup_scaffolding(namespace: String) -> Result<TokenStream> {
    let module_path = mod_path()?;
    let ffi_contract_version_ident = format_ident!("ffi_{module_path}_uniffi_contract_version");
    let namespace_upper = namespace.to_ascii_uppercase();
    let namespace_const_ident = format_ident!("UNIFFI_META_CONST_NAMESPACE_{namespace_upper}");
    let namespace_static_ident = format_ident!("UNIFFI_META_NAMESPACE_{namespace_upper}");
    let ffi_rustbuffer_alloc_ident = format_ident!("ffi_{module_path}_rustbuffer_alloc");
    let ffi_rustbuffer_from_bytes_ident = format_ident!("ffi_{module_path}_rustbuffer_from_bytes");
    let ffi_rustbuffer_free_ident = format_ident!("ffi_{module_path}_rustbuffer_free");
    let ffi_rustbuffer_reserve_ident = format_ident!("ffi_{module_path}_rustbuffer_reserve");
    let reexport_hack_ident = format_ident!("{module_path}_uniffi_reexport_hack");
    let ffi_foreign_executor_callback_set_ident =
        format_ident!("ffi_{module_path}_foreign_executor_callback_set");
    let ffi_rust_future_continuation_callback_set =
        format_ident!("ffi_{module_path}_rust_future_continuation_callback_set");
    let ffi_rust_future_poll = format_ident!("ffi_{module_path}_rust_future_poll");
    let ffi_rust_future_cancel = format_ident!("ffi_{module_path}_rust_future_cancel");
    let ffi_rust_future_free = format_ident!("ffi_{module_path}_rust_future_free");
    let ffi_rust_future_complete_fns = rust_future_complete_fns(&module_path);

    Ok(quote! {
        // Unit struct to parameterize the FfiConverter trait.
        //
        // We use FfiConverter<UniFfiTag> to handle lowering/lifting/serializing types for this crate.  See
        // https://mozilla.github.io/uniffi-rs/internals/lifting_and_lowering.html#code-generation-and-the-fficonverter-trait
        // for details.
        //
        // This is pub, since we need to access it to support external types
        #[doc(hidden)]
        pub struct UniFfiTag;

        #[allow(clippy::missing_safety_doc, missing_docs)]
        #[doc(hidden)]
        #[no_mangle]
        pub extern "C" fn #ffi_contract_version_ident() -> u32 {
            #UNIFFI_CONTRACT_VERSION
        }


        /// Export namespace metadata.
        ///
        /// See `uniffi_bindgen::macro_metadata` for how this is used.

        const #namespace_const_ident: ::uniffi::MetadataBuffer = ::uniffi::MetadataBuffer::from_code(::uniffi::metadata::codes::NAMESPACE)
            .concat_str(#module_path)
            .concat_str(#namespace);

        #[doc(hidden)]
        #[no_mangle]
        pub static #namespace_static_ident: [u8; #namespace_const_ident.size] = #namespace_const_ident.into_array();

        // Everybody gets basic buffer support, since it's needed for passing complex types over the FFI.
        //
        // See `uniffi/src/ffi/rustbuffer.rs` for documentation on these functions

        #[allow(clippy::missing_safety_doc, missing_docs)]
        #[doc(hidden)]
        #[no_mangle]
        pub extern "C" fn #ffi_rustbuffer_alloc_ident(size: i32, call_status: &mut uniffi::RustCallStatus) -> uniffi::RustBuffer {
            uniffi::ffi::uniffi_rustbuffer_alloc(size, call_status)
        }

        #[allow(clippy::missing_safety_doc, missing_docs)]
        #[doc(hidden)]
        #[no_mangle]
        pub unsafe extern "C" fn #ffi_rustbuffer_from_bytes_ident(bytes: uniffi::ForeignBytes, call_status: &mut uniffi::RustCallStatus) -> uniffi::RustBuffer {
            uniffi::ffi::uniffi_rustbuffer_from_bytes(bytes, call_status)
        }

        #[allow(clippy::missing_safety_doc, missing_docs)]
        #[doc(hidden)]
        #[no_mangle]
        pub unsafe extern "C" fn #ffi_rustbuffer_free_ident(buf: uniffi::RustBuffer, call_status: &mut uniffi::RustCallStatus) {
            uniffi::ffi::uniffi_rustbuffer_free(buf, call_status);
        }

        #[allow(clippy::missing_safety_doc, missing_docs)]
        #[doc(hidden)]
        #[no_mangle]
        pub unsafe extern "C" fn #ffi_rustbuffer_reserve_ident(buf: uniffi::RustBuffer, additional: i32, call_status: &mut uniffi::RustCallStatus) -> uniffi::RustBuffer {
            uniffi::ffi::uniffi_rustbuffer_reserve(buf, additional, call_status)
        }

        #[allow(clippy::missing_safety_doc, missing_docs)]
        #[doc(hidden)]
        #[no_mangle]
        pub extern "C" fn #ffi_foreign_executor_callback_set_ident(callback: uniffi::ffi::ForeignExecutorCallback) {
            uniffi::ffi::foreign_executor_callback_set(callback)
        }

        #[allow(clippy::missing_safety_doc, missing_docs)]
        #[doc(hidden)]
        #[no_mangle]
        pub unsafe extern "C" fn #ffi_rust_future_continuation_callback_set(callback: ::uniffi::RustFutureContinuationCallback) {
            ::uniffi::ffi::rust_future_continuation_callback_set(callback);
        }

        #[allow(clippy::missing_safety_doc, missing_docs)]
        #[doc(hidden)]
        #[no_mangle]
        pub unsafe extern "C" fn #ffi_rust_future_poll(handle: ::uniffi::RustFutureHandle, data: *const ()) {
            ::uniffi::ffi::rust_future_poll(handle, data);
        }

        #[allow(clippy::missing_safety_doc, missing_docs)]
        #[doc(hidden)]
        #[no_mangle]
        pub unsafe extern "C" fn #ffi_rust_future_cancel(handle: ::uniffi::RustFutureHandle) {
            ::uniffi::ffi::rust_future_cancel(handle)
        }

        #ffi_rust_future_complete_fns

        #[allow(clippy::missing_safety_doc, missing_docs)]
        #[doc(hidden)]
        #[no_mangle]
        pub unsafe extern "C" fn #ffi_rust_future_free(handle: ::uniffi::RustFutureHandle) {
            ::uniffi::ffi::rust_future_free(handle)
        }

        // Code to re-export the UniFFI scaffolding functions.
        //
        // Rust won't always re-export the functions from dependencies
        // ([rust-lang#50007](https://github.com/rust-lang/rust/issues/50007))
        //
        // A workaround for this is to have the dependent crate reference a function from its dependency in
        // an extern "C" function. This is clearly hacky and brittle, but at least we have some unittests
        // that check if this works (fixtures/reexport-scaffolding-macro).
        //
        // The main way we use this macro is for that contain multiple UniFFI components (libxul,
        // megazord).  The combined library has a cargo dependency for each component and calls
        // uniffi_reexport_scaffolding!() for each one.

        #[allow(missing_docs)]
        #[doc(hidden)]
        pub const fn uniffi_reexport_hack() {}

        #[doc(hidden)]
        #[macro_export]
        macro_rules! uniffi_reexport_scaffolding {
            () => {
                #[doc(hidden)]
                #[no_mangle]
                pub extern "C" fn #reexport_hack_ident() {
                    $crate::uniffi_reexport_hack()
                }
            };
        }

        // A trait that's in our crate for our external wrapped types to implement.
        #[allow(unused)]
        #[doc(hidden)]
        pub trait UniffiCustomTypeConverter {
            type Builtin;
            fn into_custom(val: Self::Builtin) -> uniffi::Result<Self> where Self: Sized;
            fn from_custom(obj: Self) -> Self::Builtin;
        }
    })
}

/// Generates the rust_future_complete functions
///
/// The foreign side uses a type-erased `RustFutureHandle` to interact with futures, which presents
/// a problem when completing them.  What type does the future output?
///
/// Handle this by using some brute-force monomorphization.  For each possible ffi type, we
/// generate a rust_future_complete function.  The bindings code is responsible for calling the one
/// corresponds the scaffolding function that created the `RustFutureHandle`.
///
/// This introduces safety issues, but we do get some type checking.  If the bindings code calls
/// the wrong rust_future_complete function, they should get an unexpected return type, which
/// hopefully will result in a compile-time error.
fn rust_future_complete_fns(module_path: &str) -> TokenStream {
    let fn_info = [
        (
            quote! { u8 },
            format_ident!("ffi_{module_path}_rust_future_complete_u8"),
        ),
        (
            quote! { i8 },
            format_ident!("ffi_{module_path}_rust_future_complete_i8"),
        ),
        (
            quote! { u16 },
            format_ident!("ffi_{module_path}_rust_future_complete_u16"),
        ),
        (
            quote! { i16 },
            format_ident!("ffi_{module_path}_rust_future_complete_i16"),
        ),
        (
            quote! { u32 },
            format_ident!("ffi_{module_path}_rust_future_complete_u32"),
        ),
        (
            quote! { i32 },
            format_ident!("ffi_{module_path}_rust_future_complete_i32"),
        ),
        (
            quote! { u64 },
            format_ident!("ffi_{module_path}_rust_future_complete_u64"),
        ),
        (
            quote! { i64 },
            format_ident!("ffi_{module_path}_rust_future_complete_i64"),
        ),
        (
            quote! { f32 },
            format_ident!("ffi_{module_path}_rust_future_complete_f32"),
        ),
        (
            quote! { f64 },
            format_ident!("ffi_{module_path}_rust_future_complete_f64"),
        ),
        (
            quote! { *const ::std::ffi::c_void },
            format_ident!("ffi_{module_path}_rust_future_complete_pointer"),
        ),
        (
            quote! { ::uniffi::RustBuffer },
            format_ident!("ffi_{module_path}_rust_future_complete_rust_buffer"),
        ),
        (
            quote! { () },
            format_ident!("ffi_{module_path}_rust_future_complete_void"),
        ),
    ];

    let types = fn_info.iter().map(|i| &i.0);
    let names = fn_info.iter().map(|i| &i.1);

    quote! {
        #(
            #[allow(clippy::missing_safety_doc, missing_docs)]
            #[doc(hidden)]
            #[no_mangle]
            pub unsafe extern "C" fn #names(
                handle: ::uniffi::RustFutureHandle,
                out_status: &mut ::uniffi::RustCallStatus
            ) -> #types {
                ::uniffi::ffi::rust_future_complete(handle, out_status)
            }
        )*
    }
}
