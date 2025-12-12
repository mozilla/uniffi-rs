/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use proc_macro2::{Ident, TokenStream};
use quote::quote;
use std::iter;

use super::attributes::AsyncRuntime;
use crate::{
    ffiops,
    fnsig::{FnKind, FnSignature},
};

pub(super) fn gen_fn_scaffolding(
    sig: FnSignature,
    ar: Option<&AsyncRuntime>,
    udl_mode: bool,
) -> syn::Result<TokenStream> {
    if sig.receiver.is_some() {
        return Err(syn::Error::new(
            sig.span,
            "Unexpected self param (Note: uniffi::export must be used on the impl block, not its containing fn's)"
        ));
    }
    if !sig.is_async {
        if let Some(async_runtime) = ar {
            return Err(syn::Error::new_spanned(
                async_runtime,
                "this attribute is only allowed on async functions",
            ));
        }
    }
    let metadata_items = (!udl_mode).then(|| {
        sig.metadata_items()
            .unwrap_or_else(syn::Error::into_compile_error)
    });
    let scaffolding_func = gen_ffi_function(&sig, ar, udl_mode, None)?;
    Ok(quote! {
        #scaffolding_func
        #metadata_items
    })
}

pub(super) fn gen_constructor_scaffolding(
    sig: FnSignature,
    ar: Option<&AsyncRuntime>,
    udl_mode: bool,
) -> syn::Result<TokenStream> {
    if sig.receiver.is_some() {
        return Err(syn::Error::new(
            sig.span,
            "constructors must not have a self parameter",
        ));
    }
    let metadata_items = (!udl_mode).then(|| {
        sig.metadata_items()
            .unwrap_or_else(syn::Error::into_compile_error)
    });
    let scaffolding_func = gen_ffi_function(&sig, ar, udl_mode, None)?;
    Ok(quote! {
        #scaffolding_func
        #metadata_items
    })
}

pub(super) fn gen_method_scaffolding(
    sig: FnSignature,
    ar: Option<&AsyncRuntime>,
    udl_mode: bool,
    use_trait: Option<&syn::Path>,
) -> syn::Result<TokenStream> {
    let scaffolding_func = if sig.receiver.is_none() {
        return Err(syn::Error::new(
            sig.span,
            "associated functions are not currently supported",
        ));
    } else {
        gen_ffi_function(&sig, ar, udl_mode, use_trait)?
    };

    let metadata_items = (!udl_mode).then(|| {
        sig.metadata_items()
            .unwrap_or_else(syn::Error::into_compile_error)
    });
    Ok(quote! {
        #scaffolding_func
        #metadata_items
    })
}

// Code used to generate scaffolding functions
struct ScaffoldingBits {
    /// Parameter names for the scaffolding function
    param_names: Vec<TokenStream>,
    /// Parameter types for the scaffolding function
    param_types: Vec<TokenStream>,
    /// Lift closure.  See `FnSignature::lift_closure` for an explanation of this.
    lift_closure: TokenStream,
    /// Expression to call the Rust function after a successful lift.
    rust_fn_call: TokenStream,
    /// Convert the result of `rust_fn_call`, stored in a variable named `uniffi_result` into its final value.
    /// This is used to do things like error conversion / Arc wrapping
    convert_result: TokenStream,
}

impl ScaffoldingBits {
    fn new_for_function(sig: &FnSignature, udl_mode: bool) -> Self {
        let ident = &sig.ident;
        let call_params = sig.rust_call_params(false);
        let rust_fn_call = quote! { #ident(#call_params) };
        // UDL mode adds an extra conversion (#1749)
        let convert_result = if udl_mode && sig.looks_like_result {
            quote! { uniffi_result.map_err(::std::convert::Into::into) }
        } else {
            quote! { uniffi_result }
        };

        Self {
            param_names: sig.scaffolding_param_names().collect(),
            param_types: sig.scaffolding_param_types().collect(),
            lift_closure: sig.lift_closure(None),
            rust_fn_call,
            convert_result,
        }
    }

    fn new_for_method(
        sig: &FnSignature,
        self_ident: &Ident,
        is_trait: bool,
        udl_mode: bool,
    ) -> Self {
        let ident = &sig.ident;
        let self_type = if is_trait {
            quote! { dyn #self_ident }
        } else {
            quote! { #self_ident }
        };

        let ref_type = ffiops::lift_ref_type(&self_type);
        let lift_type = ffiops::lift_type(&ref_type);
        let try_lift = ffiops::try_lift(&ref_type);

        let lift_closure = sig.lift_closure(Some(quote! {
            match #try_lift(uniffi_self_lowered) {
                ::std::result::Result::Ok(v) => v,
                ::std::result::Result::Err(e) => {
                    return ::std::result::Result::Err(("self", e));
                }
            }
        }));
        let call_params = sig.rust_call_params(true);
        let rust_fn_call = quote! { uniffi_args.0.#ident(#call_params) };
        // UDL mode adds an extra conversion (#1749)
        let convert_result = if udl_mode && sig.looks_like_result {
            quote! { uniffi_result .map_err(::std::convert::Into::into) }
        } else {
            quote! { uniffi_result }
        };

        Self {
            param_names: iter::once(quote! { uniffi_self_lowered })
                .chain(sig.scaffolding_param_names())
                .collect(),
            param_types: iter::once(quote! { #lift_type })
                .chain(sig.scaffolding_param_types())
                .collect(),
            lift_closure,
            rust_fn_call,
            convert_result,
        }
    }

    fn new_for_constructor(sig: &FnSignature, self_ident: &Ident, udl_mode: bool) -> Self {
        let ident = &sig.ident;
        let call_params = sig.rust_call_params(false);
        let rust_fn_call = quote! { #self_ident::#ident(#call_params) };
        // UDL mode adds extra conversions (#1749)
        let convert_result = match (udl_mode, sig.looks_like_result) {
            // For UDL
            (true, false) => quote! { ::std::sync::Arc::new(uniffi_result) },
            (true, true) => {
                quote! { uniffi_result.map(::std::sync::Arc::new).map_err(::std::convert::Into::into) }
            }
            (false, _) => quote! { uniffi_result },
        };

        Self {
            param_names: sig.scaffolding_param_names().collect(),
            param_types: sig.scaffolding_param_types().collect(),
            lift_closure: sig.lift_closure(None),
            rust_fn_call,
            convert_result,
        }
    }
}

/// Generate a scaffolding function
///
/// `pre_fn_call` is the statements that we should execute before the rust call
/// `rust_fn` is the Rust function to call.
pub(super) fn gen_ffi_function(
    sig: &FnSignature,
    ar: Option<&AsyncRuntime>,
    udl_mode: bool,
    use_trait: Option<&syn::Path>,
) -> syn::Result<TokenStream> {
    let bits = match &sig.kind {
        FnKind::Function => ScaffoldingBits::new_for_function(sig, udl_mode),
        FnKind::Method { self_ident, .. } => {
            ScaffoldingBits::new_for_method(sig, self_ident, false, udl_mode)
        }
        FnKind::TraitMethod { self_ident, .. } => {
            ScaffoldingBits::new_for_method(sig, self_ident, true, udl_mode)
        }
        FnKind::Constructor { self_ident, .. } => {
            ScaffoldingBits::new_for_constructor(sig, self_ident, udl_mode)
        }
    };

    let ffi_ident = sig.scaffolding_fn_ident()?;
    let use_trait = use_trait.map(|tr| quote! { use #tr; });

    Ok(if !sig.is_async {
        let mut tokens = TokenStream::default();
        tokens.extend(scaffolding_fn_legacy_ffi(
            sig,
            &ffi_ident,
            &bits,
            use_trait.as_ref(),
        ));
        #[cfg(feature = "pointer-ffi")]
        tokens.extend(scaffolding_fn_pointer_ffi(
            sig,
            &ffi_ident,
            &bits,
            use_trait.as_ref(),
        ));
        tokens
    } else {
        let rust_fn_call = &bits.rust_fn_call;
        let future_expr = match ar {
            Some(AsyncRuntime::Tokio(_)) => {
                quote! { ::uniffi::deps::async_compat::Compat::new(#rust_fn_call) }
            }
            None => rust_fn_call.clone(),
        };

        let mut tokens = TokenStream::default();
        tokens.extend(scaffolding_async_fn_legacy_ffi(
            sig,
            &ffi_ident,
            &bits,
            &future_expr,
        ));
        #[cfg(feature = "pointer-ffi")]
        tokens.extend(scaffolding_async_fn_pointer_ffi(
            sig,
            &ffi_ident,
            &bits,
            &future_expr,
        ));
        tokens
    })
}

fn scaffolding_fn_legacy_ffi(
    sig: &FnSignature,
    ident: &Ident,
    bits: &ScaffoldingBits,
    use_trait: Option<&TokenStream>,
) -> TokenStream {
    let ScaffoldingBits {
        param_names,
        param_types,
        lift_closure,
        rust_fn_call,
        convert_result,
    } = bits;

    let ffi_fn_name = ident.to_string();
    let return_ty = &sig.return_ty;
    let ffi_return_ty = ffiops::lower_return_type(return_ty);
    let lower_return = ffiops::lower_return(return_ty);
    let handle_failed_lift = ffiops::lower_return_handle_failed_lift(return_ty);

    quote! {
        #[doc(hidden)]
        #[unsafe(no_mangle)]
        pub extern "C" fn #ident(
            #(#param_names: #param_types,)*
            call_status: &mut ::uniffi::RustCallStatus,
        ) -> #ffi_return_ty {
            #use_trait
            ::uniffi::deps::trace!("calling: {}", #ffi_fn_name);
            let uniffi_lift_args = #lift_closure;
            ::uniffi::rust_call(call_status, || {
                let result = match uniffi_lift_args() {
                    ::std::result::Result::Ok(uniffi_args) => {
                        ::uniffi::deps::trace!("lift_args success: {}", #ffi_fn_name);
                        let uniffi_result = #rust_fn_call;
                        ::uniffi::deps::trace!("call success: {}", #ffi_fn_name);
                        let uniffi_lowered_return = #lower_return(#convert_result);
                        ::uniffi::deps::trace!("lower_return success: {}", #ffi_fn_name);
                        uniffi_lowered_return
                    }
                    ::std::result::Result::Err((arg_name, error)) => {
                        ::uniffi::deps::trace!("lift_args error: {}", #ffi_fn_name);
                        #handle_failed_lift(::uniffi::LiftArgsError { arg_name, error} )
                    },
                };
                result
            })
        }
    }
}

fn scaffolding_async_fn_legacy_ffi(
    sig: &FnSignature,
    ident: &Ident,
    bits: &ScaffoldingBits,
    future_expr: &TokenStream,
) -> TokenStream {
    let ScaffoldingBits {
        param_names,
        param_types,
        lift_closure,
        convert_result,
        ..
    } = bits;

    let ffi_fn_name = ident.to_string();
    let return_ty = &sig.return_ty;

    quote! {
        #[doc(hidden)]
        #[unsafe(no_mangle)]
        pub extern "C" fn #ident(#(#param_names: #param_types,)*) -> ::uniffi::Handle {
            ::uniffi::deps::trace!("calling: {}", #ffi_fn_name);
            let uniffi_lifted_args = (#lift_closure)();
            ::uniffi::rust_future_new::<_, #return_ty, _>(
                async move {
                    match uniffi_lifted_args {
                        ::std::result::Result::Ok(uniffi_args) => {
                            let uniffi_result = #future_expr.await;
                            Ok(#convert_result)
                        }
                        ::std::result::Result::Err((arg_name, error)) => {
                            Err(::uniffi::LiftArgsError { arg_name, error })
                        },
                    }
                },
                crate::UniFfiTag
            )
        }
    }
}

#[cfg(feature = "pointer-ffi")]
fn scaffolding_fn_pointer_ffi(
    sig: &FnSignature,
    ident: &Ident,
    bits: &ScaffoldingBits,
    use_trait: Option<&TokenStream>,
) -> TokenStream {
    let ScaffoldingBits {
        param_names,
        param_types,
        lift_closure,
        rust_fn_call,
        convert_result,
    } = bits;

    let ffi_fn_name = uniffi_meta::pointer_ffi_symbol_name(&ident.to_string());
    let ident = Ident::new(&ffi_fn_name, proc_macro2::Span::call_site());
    let return_ty = &sig.return_ty;
    let ffi_return_ty = ffiops::lower_return_type(return_ty);
    let lower_return = ffiops::lower_return(return_ty);
    let handle_failed_lift = ffiops::lower_return_handle_failed_lift(return_ty);

    quote! {
        #[doc(hidden)]
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn #ident(uniffi_ffi_buffer: *mut u8) {
            #use_trait
            ::uniffi::deps::trace!("calling(ptr): {}", #ffi_fn_name);

            let mut uniffi_args_buf = ::std::slice::from_raw_parts(
                uniffi_ffi_buffer,
                ::uniffi::ffi_buffer_size!((#(#param_types),*)),
            );
            #(
                let #param_names = <#param_types as ::uniffi::FfiSerialize>::read(&mut uniffi_args_buf);
            )*

            let uniffi_lift_args = #lift_closure;

            let mut uniffi_call_status = <::uniffi::RustCallStatus as ::std::default::Default>::default();
            let uniffi_return_value = ::uniffi::rust_call(&mut uniffi_call_status, || {
                match uniffi_lift_args() {
                    ::std::result::Result::Ok(uniffi_args) => {
                        ::uniffi::deps::trace!("lift_args success: {}", #ffi_fn_name);
                        let uniffi_result = #rust_fn_call;
                        ::uniffi::deps::trace!("call success: {}", #ffi_fn_name);
                        let uniffi_lowered_return = #lower_return(#convert_result);
                        ::uniffi::deps::trace!("lower_return success: {}", #ffi_fn_name);
                        uniffi_lowered_return
                    }
                    ::std::result::Result::Err((arg_name, error)) => {
                        ::uniffi::deps::trace!("lift_args error: {}", #ffi_fn_name);
                        #handle_failed_lift(::uniffi::LiftArgsError { arg_name, error} )
                    },
                }
            });

            //  Reuse the argument buffer to return the value.  This is safe, since we're done
            //  reading from the arg buffer.
            let mut uniffi_return_buf = ::std::slice::from_raw_parts_mut(
                uniffi_ffi_buffer,
                ::uniffi::ffi_buffer_size!((::uniffi::RustCallStatus, #ffi_return_ty)),
            );
            <::uniffi::RustCallStatus as ::uniffi::FfiSerialize>::write(&mut uniffi_return_buf, uniffi_call_status);
            <#ffi_return_ty as ::uniffi::FfiSerialize>::write(&mut uniffi_return_buf, uniffi_return_value);
        }
    }
}

#[cfg(feature = "pointer-ffi")]
fn scaffolding_async_fn_pointer_ffi(
    sig: &FnSignature,
    ident: &Ident,
    bits: &ScaffoldingBits,
    future_expr: &TokenStream,
) -> TokenStream {
    let ScaffoldingBits {
        param_names,
        param_types,
        lift_closure,
        convert_result,
        ..
    } = bits;

    let ffi_fn_name = uniffi_meta::pointer_ffi_symbol_name(&ident.to_string());
    let ident = Ident::new(&ffi_fn_name, proc_macro2::Span::call_site());
    let return_ty = &sig.return_ty;

    quote! {
        #[doc(hidden)]
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn #ident(uniffi_ffi_buffer: *mut u8) {
            ::uniffi::deps::trace!("calling: {}", #ffi_fn_name);

            let mut uniffi_args_buf = ::std::slice::from_raw_parts(
                uniffi_ffi_buffer,
                ::uniffi::ffi_buffer_size!((#(#param_types),*)),
            );
            #(
                let #param_names = <#param_types as ::uniffi::FfiSerialize>::read(&mut uniffi_args_buf);
            )*

            let uniffi_lifted_args = (#lift_closure)();
            ::uniffi::pointer_ffi::rust_future_new::<_, #return_ty, _>(
                async move {
                    match uniffi_lifted_args {
                        ::std::result::Result::Ok(uniffi_args) => {
                            let uniffi_result = #future_expr.await;
                            Ok(#convert_result)
                        }
                        ::std::result::Result::Err((arg_name, error)) => {
                            Err(::uniffi::LiftArgsError { arg_name, error })
                        },
                    }
                },
                crate::UniFfiTag,
                uniffi_ffi_buffer
            );
        }
    }
}
