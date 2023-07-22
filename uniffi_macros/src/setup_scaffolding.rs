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
    let ffi_contract_version_ident = format_ident!("ffi_{namespace}_uniffi_contract_version");
    let namespace_upper = namespace.to_ascii_uppercase();
    let namespace_const_ident = format_ident!("UNIFFI_META_CONST_NAMESPACE_{namespace_upper}");
    let namespace_static_ident = format_ident!("UNIFFI_META_NAMESPACE_{namespace_upper}");
    let ffi_rustbuffer_alloc_ident = format_ident!("ffi_{namespace}_rustbuffer_alloc");
    let ffi_rustbuffer_from_bytes_ident = format_ident!("ffi_{namespace}_rustbuffer_from_bytes");
    let ffi_rustbuffer_free_ident = format_ident!("ffi_{namespace}_rustbuffer_free");
    let ffi_rustbuffer_reserve_ident = format_ident!("ffi_{namespace}_rustbuffer_reserve");
    let reexport_hack_ident = format_ident!("{namespace}_uniffi_reexport_hack");

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
        trait UniffiCustomTypeConverter {
            type Builtin;
            fn into_custom(val: Self::Builtin) -> uniffi::Result<Self> where Self: Sized;
            fn from_custom(obj: Self) -> Self::Builtin;
        }
    })
}
