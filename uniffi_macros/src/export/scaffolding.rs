/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote};
use syn::{FnArg, Pat, ReturnType, Signature};

pub(super) fn gen_fn_scaffolding(
    sig: &Signature,
    mod_path: &[String],
    checksum: u16,
) -> TokenStream {
    let name = &sig.ident;
    let name_s = name.to_string();
    let ffi_name = Ident::new(
        &uniffi_meta::fn_ffi_symbol_name(mod_path, &name_s, checksum),
        Span::call_site(),
    );

    let (params, args): (Vec<_>, Vec<_>) = sig
        .inputs
        .iter()
        .enumerate()
        .map(|(i, arg)| match arg {
            FnArg::Receiver(receiver) => {
                let param = quote! { &self };
                let arg = syn::Error::new_spanned(
                    receiver,
                    "methods are not yet supported by uniffi::export",
                )
                .into_compile_error();

                (param, arg)
            }
            FnArg::Typed(pat_ty) => {
                let ty = &pat_ty.ty;
                let name = format_ident!("arg{i}");

                let param = quote! { #name: <#ty as ::uniffi::FfiConverter>::FfiType };

                let panic_fmt = match &*pat_ty.pat {
                    Pat::Ident(i) => {
                        format!("Failed to convert arg '{}': {{}}", i.ident)
                    }
                    _ => {
                        format!("Failed to convert arg #{i}: {{}}")
                    }
                };
                let arg = quote! {
                    <#ty as ::uniffi::FfiConverter>::try_lift(#name).unwrap_or_else(|err| {
                        ::std::panic!(#panic_fmt, err)
                    })
                };

                (param, arg)
            }
        })
        .unzip();

    let fn_call = quote! {
        #name(#(#args),*)
    };

    // FIXME(jplatte): Use an extra trait implemented for `T: FfiConverter` as
    // well as `()` so no different codegen is needed?
    let (output, return_expr);
    match &sig.output {
        ReturnType::Default => {
            output = None;
            return_expr = fn_call;
        }
        ReturnType::Type(_, ty) => {
            output = Some(quote! {
                -> <#ty as ::uniffi::FfiConverter>::FfiType
            });
            return_expr = quote! {
                <#ty as ::uniffi::FfiConverter>::lower(#fn_call)
            };
        }
    }

    quote! {
        #[doc(hidden)]
        #[no_mangle]
        pub extern "C" fn #ffi_name(
            #(#params,)*
            call_status: &mut ::uniffi::RustCallStatus,
        ) #output {
            ::uniffi::deps::log::debug!(#name_s);
            ::uniffi::call_with_output(call_status, || {
                #return_expr
            })
        }
    }
}
