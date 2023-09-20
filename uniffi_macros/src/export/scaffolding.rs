/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use proc_macro2::{Ident, TokenStream};
use quote::quote;
use std::iter;

use super::attributes::{AsyncRuntime, ExportAttributeArguments};
use crate::fnsig::{FnKind, FnSignature, NamedArg};

pub(super) fn gen_fn_scaffolding(
    sig: FnSignature,
    arguments: &ExportAttributeArguments,
    udl_mode: bool,
) -> syn::Result<TokenStream> {
    if sig.receiver.is_some() {
        return Err(syn::Error::new(
            sig.span,
            "Unexpected self param (Note: uniffi::export must be used on the impl block, not its containing fn's)"
        ));
    }
    if !sig.is_async {
        if let Some(async_runtime) = &arguments.async_runtime {
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
    let scaffolding_func = gen_ffi_function(&sig, arguments, udl_mode)?;
    Ok(quote! {
        #scaffolding_func
        #metadata_items
    })
}

pub(super) fn gen_constructor_scaffolding(
    sig: FnSignature,
    arguments: &ExportAttributeArguments,
    udl_mode: bool,
) -> syn::Result<TokenStream> {
    if sig.receiver.is_some() {
        return Err(syn::Error::new(
            sig.span,
            "constructors must not have a self parameter",
        ));
    }
    if sig.is_async {
        return Err(syn::Error::new(sig.span, "constructors can't be async"));
    }
    let metadata_items = (!udl_mode).then(|| {
        sig.metadata_items()
            .unwrap_or_else(syn::Error::into_compile_error)
    });
    let scaffolding_func = gen_ffi_function(&sig, arguments, udl_mode)?;
    Ok(quote! {
        #scaffolding_func
        #metadata_items
    })
}

pub(super) fn gen_method_scaffolding(
    sig: FnSignature,
    arguments: &ExportAttributeArguments,
    udl_mode: bool,
) -> syn::Result<TokenStream> {
    let scaffolding_func = if sig.receiver.is_none() {
        return Err(syn::Error::new(
            sig.span,
            "associated functions are not currently supported",
        ));
    } else {
        gen_ffi_function(&sig, arguments, udl_mode)?
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

// Pieces of code for the scaffolding function
struct ScaffoldingBits {
    /// Parameters for the scaffolding function
    params: Vec<TokenStream>,
    /// Statements to execute before `rust_fn_call`
    pre_fn_call: TokenStream,
    /// Tokenstream for the call to the actual Rust function
    rust_fn_call: TokenStream,
}

impl ScaffoldingBits {
    fn new_for_function(sig: &FnSignature, udl_mode: bool) -> Self {
        let ident = &sig.ident;
        let params: Vec<_> = sig.args.iter().map(NamedArg::scaffolding_param).collect();
        let param_lifts = sig.lift_exprs();
        let simple_rust_fn_call = quote! { #ident(#(#param_lifts,)*) };
        let rust_fn_call = if udl_mode && sig.looks_like_result {
            quote! { #simple_rust_fn_call.map_err(::std::convert::Into::into) }
        } else {
            simple_rust_fn_call
        };

        Self {
            params,
            pre_fn_call: quote! {},
            rust_fn_call,
        }
    }

    fn new_for_method(
        sig: &FnSignature,
        self_ident: &Ident,
        is_trait: bool,
        udl_mode: bool,
    ) -> Self {
        let ident = &sig.ident;
        let ffi_converter = if is_trait {
            quote! {
                <::std::sync::Arc<dyn #self_ident> as ::uniffi::FfiConverter<crate::UniFfiTag>>
            }
        } else {
            quote! {
                <::std::sync::Arc<#self_ident> as ::uniffi::FfiConverter<crate::UniFfiTag>>
            }
        };
        let params: Vec<_> = iter::once(quote! { uniffi_self_lowered: #ffi_converter::FfiType })
            .chain(sig.scaffolding_params())
            .collect();
        let param_lifts = sig.lift_exprs();
        let simple_rust_fn_call = quote! { uniffi_self.#ident(#(#param_lifts,)*) };
        let rust_fn_call = if udl_mode && sig.looks_like_result {
            quote! { #simple_rust_fn_call.map_err(::std::convert::Into::into) }
        } else {
            simple_rust_fn_call
        };
        let return_ffi_converter = sig.return_ffi_converter();

        Self {
            params,
            pre_fn_call: quote! {
                let uniffi_self = match #ffi_converter::try_lift(uniffi_self_lowered) {
                    Ok(v) => v,
                    Err(e) => return Err(#return_ffi_converter::handle_failed_lift("self", e)),
                };
            },
            rust_fn_call,
        }
    }

    fn new_for_constructor(sig: &FnSignature, self_ident: &Ident, udl_mode: bool) -> Self {
        let ident = &sig.ident;
        let params: Vec<_> = sig.args.iter().map(NamedArg::scaffolding_param).collect();
        let param_lifts = sig.lift_exprs();
        let simple_rust_fn_call = quote! { #self_ident::#ident(#(#param_lifts,)*) };
        let rust_fn_call = match (udl_mode, sig.looks_like_result) {
            // For UDL
            (true, false) => quote! { ::std::sync::Arc::new(#simple_rust_fn_call) },
            (true, true) => {
                quote! { #simple_rust_fn_call.map(::std::sync::Arc::new).map_err(::std::convert::Into::into) }
            }
            (false, _) => simple_rust_fn_call,
        };

        Self {
            params,
            pre_fn_call: quote! {},
            rust_fn_call,
        }
    }
}

/// Generate a scaffolding function
///
/// `pre_fn_call` is the statements that we should execute before the rust call
/// `rust_fn` is the Rust function to call.
fn gen_ffi_function(
    sig: &FnSignature,
    arguments: &ExportAttributeArguments,
    udl_mode: bool,
) -> syn::Result<TokenStream> {
    let ScaffoldingBits {
        params,
        pre_fn_call,
        rust_fn_call,
    } = match &sig.kind {
        FnKind::Function => ScaffoldingBits::new_for_function(sig, udl_mode),
        FnKind::Method { self_ident } => {
            ScaffoldingBits::new_for_method(sig, self_ident, false, udl_mode)
        }
        FnKind::TraitMethod { self_ident, .. } => {
            ScaffoldingBits::new_for_method(sig, self_ident, true, udl_mode)
        }
        FnKind::Constructor { self_ident } => {
            ScaffoldingBits::new_for_constructor(sig, self_ident, udl_mode)
        }
    };
    // Scaffolding functions are logically `pub`, but we don't use that in UDL mode since UDL has
    // historically not required types to be `pub`
    let vis = match udl_mode {
        false => quote! { pub },
        true => quote! {},
    };

    let ffi_ident = sig.scaffolding_fn_ident()?;
    let name = &sig.name;
    let return_ty = &sig.return_ty;

    Ok(if !sig.is_async {
        quote! {
            #[doc(hidden)]
            #[no_mangle]
            #vis extern "C" fn #ffi_ident(
                #(#params,)*
                call_status: &mut ::uniffi::RustCallStatus,
            ) -> <#return_ty as ::uniffi::FfiConverter<crate::UniFfiTag>>::ReturnType {
                ::uniffi::deps::log::debug!(#name);
                ::uniffi::rust_call(call_status, || {
                    #pre_fn_call
                    <#return_ty as ::uniffi::FfiConverter<crate::UniFfiTag>>::lower_return(
                        #rust_fn_call
                    )
                })
            }
        }
    } else {
        let mut future_expr = rust_fn_call;
        if matches!(arguments.async_runtime, Some(AsyncRuntime::Tokio(_))) {
            future_expr = quote! { ::uniffi::deps::async_compat::Compat::new(#future_expr) }
        }

        quote! {
            #[doc(hidden)]
            #[no_mangle]
            #vis extern "C" fn #ffi_ident(
                #(#params,)*
                uniffi_executor_handle: ::uniffi::ForeignExecutorHandle,
                uniffi_callback: <#return_ty as ::uniffi::FfiConverter<crate::UniFfiTag>>::FutureCallback,
                uniffi_callback_data: *const (),
                uniffi_call_status: &mut ::uniffi::RustCallStatus,
            ) {
                ::uniffi::deps::log::debug!(#name);
                ::uniffi::rust_call(uniffi_call_status, || {
                    #pre_fn_call;
                    let uniffi_rust_future = ::uniffi::RustFuture::<_, #return_ty, crate::UniFfiTag>::new(
                        #future_expr,
                        uniffi_executor_handle,
                        uniffi_callback,
                        uniffi_callback_data
                    );
                    uniffi_rust_future.wake();
                    Ok(())
                });
            }
        }
    })
}
