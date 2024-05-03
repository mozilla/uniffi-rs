/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Functions, types and expressions to handle FFI operations.
//!
//! This module leverages the various traits defined in `uniffi_core::ffi_converter_traits` to provide functionality to the rest of `uniffi_macros`.
//! Keeping this layer separate makes it easier to handle changes to those traits.
//! See `uniffi_core::ffi_converter_traits` for the meaning of these functions.

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

// Lower type
pub fn lower_type(ty: impl ToTokens) -> TokenStream {
    quote! {
        <#ty as ::uniffi::Lower<crate::UniFfiTag>>::FfiType
    }
}

// Lower function
pub fn lower(ty: impl ToTokens) -> TokenStream {
    quote! {
        <#ty as ::uniffi::Lower<crate::UniFfiTag>>::lower
    }
}

// Lift type
pub fn lift_type(ty: impl ToTokens) -> TokenStream {
    quote! {
        <#ty as ::uniffi::Lift<crate::UniFfiTag>>::FfiType
    }
}

// Lift function
pub fn try_lift(ty: impl ToTokens) -> TokenStream {
    quote! {
        <#ty as ::uniffi::Lift<crate::UniFfiTag>>::try_lift
    }
}

/// Write function
pub fn write(ty: impl ToTokens) -> TokenStream {
    quote! {
        <#ty as ::uniffi::Lower<crate::UniFfiTag>>::write
    }
}

/// Read function
pub fn try_read(ty: impl ToTokens) -> TokenStream {
    quote! {
        <#ty as ::uniffi::Lift<crate::UniFfiTag>>::try_read
    }
}

/// Lower return type
pub fn lower_return_type(ty: impl ToTokens) -> TokenStream {
    quote! {
        <#ty as ::uniffi::LowerReturn<crate::UniFfiTag>>::ReturnType
    }
}

/// Lower return function
pub fn lower_return(ty: impl ToTokens) -> TokenStream {
    quote! {
        <#ty as ::uniffi::LowerReturn<crate::UniFfiTag>>::lower_return
    }
}

/// Lower error function
pub fn lower_error(ty: impl ToTokens) -> TokenStream {
    quote! {
        <#ty as ::uniffi::LowerError<crate::UniFfiTag>>::lower_error
    }
}

/// Lift return type
pub fn lift_return_type(ty: impl ToTokens) -> TokenStream {
    quote! {
        <#ty as ::uniffi::LiftReturn<crate::UniFfiTag>>::ReturnType
    }
}

/// Lift foreign return function
pub fn lift_foreign_return(ty: impl ToTokens) -> TokenStream {
    quote! {
        <#ty as ::uniffi::LiftReturn<crate::UniFfiTag>>::lift_foreign_return
    }
}

/// Handle failed lift function
pub fn lower_return_handle_failed_lift(ty: impl ToTokens) -> TokenStream {
    quote! {
        <#ty as ::uniffi::LowerReturn<crate::UniFfiTag>>::handle_failed_lift
    }
}

/// LiftRef type
pub fn lift_ref_type(ty: impl ToTokens) -> TokenStream {
    quote! { <#ty as ::uniffi::LiftRef<crate::UniFfiTag>>::LiftType }
}

/// Lower into rust buffer function
pub fn lower_into_rust_buffer(ty: impl ToTokens) -> TokenStream {
    quote! {
        <#ty as ::uniffi::Lower<crate::UniFfiTag>>::lower_into_rust_buffer
    }
}

/// Lift from rust buffer function
pub fn try_lift_from_rust_buffer(ty: impl ToTokens) -> TokenStream {
    quote! {
        <#ty as ::uniffi::Lift<crate::UniFfiTag>>::try_lift_from_rust_buffer
    }
}

/// Expression for the TYPE_ID_META value for a type
pub fn type_id_meta(ty: impl ToTokens) -> TokenStream {
    quote! {
        <#ty as ::uniffi::TypeId<crate::UniFfiTag>>::TYPE_ID_META
    }
}
