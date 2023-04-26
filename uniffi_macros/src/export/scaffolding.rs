/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::{spanned::Spanned, FnArg, Pat};

use super::{
    attributes::{AsyncRuntime, ExportAttributeArguments},
    item::Signature,
};
use crate::util::{create_metadata_items, ident_to_string, try_metadata_value_from_usize};

pub(super) fn gen_fn_scaffolding(
    sig: &Signature,
    mod_path: &str,
    arguments: &ExportAttributeArguments,
) -> syn::Result<TokenStream> {
    let name = &sig.ident;
    let name_s = ident_to_string(name);

    let ffi_ident = Ident::new(
        &uniffi_meta::fn_ffi_symbol_name(mod_path, &name_s),
        Span::call_site(),
    );

    const ERROR_MSG: &str =
        "uniffi::export must be used on the impl block, not its containing fn's";
    let mut bits = ScaffoldingBits::new();
    bits.collect_params(&sig.inputs, ERROR_MSG);
    bits.set_rust_fn_call(quote! { #name });
    let metadata_var = bits.gen_function_meta_static_var(sig, mod_path)?;
    let scaffolding_func = gen_ffi_function(sig, ffi_ident, &bits, arguments);
    Ok(quote! {
        #scaffolding_func
        #metadata_var
    })
}

pub(super) fn gen_method_scaffolding(
    sig: &Signature,
    mod_path: &str,
    self_ident: &Ident,
    arguments: &ExportAttributeArguments,
) -> syn::Result<TokenStream> {
    let ident = &sig.ident;
    let name_s = ident_to_string(ident);

    let ffi_ident = Ident::new(
        &uniffi_meta::method_fn_symbol_name(mod_path, &ident_to_string(self_ident), &name_s),
        Span::call_site(),
    );

    const RECEIVER_ERROR: &str = "unreachable: only first parameter can be method receiver";
    let bits = match sig.inputs.first() {
        // Method calls
        Some(arg) if is_receiver(arg) => {
            let ffi_converter = quote! {
                <::std::sync::Arc<#self_ident> as ::uniffi::FfiConverter<crate::UniFfiTag>>
            };
            let mut bits = ScaffoldingBits::new();
            // The first scaffolding parameter is `this` -- the lowered value for `self`
            bits.add_self_param(quote! { this: #ffi_converter::FfiType });
            // This is followed by the method arguments
            bits.collect_params(sig.inputs.iter().skip(1), RECEIVER_ERROR);
            // To call the method:
            //   - lift the `this` param to get the object
            //   - Add `.#ident` to get the method
            bits.set_rust_fn_call(quote! {
                #ffi_converter::try_lift(this).unwrap_or_else(|err| {
                    ::std::panic!("Failed to convert arg 'self': {}", err)
                }).#ident
            });
            bits
        }
        // Associated functions
        _ => {
            return Err(syn::Error::new_spanned(
                &sig.ident,
                "associated functions are not currently supported",
            ))
        }
    };

    let metadata_var = bits.gen_method_meta_static_var(self_ident, sig, mod_path);
    let scaffolding_func = gen_ffi_function(sig, ffi_ident, &bits, arguments);
    Ok(quote! {
        #scaffolding_func
        #metadata_var
    })
}

fn is_receiver(fn_arg: &FnArg) -> bool {
    match fn_arg {
        FnArg::Receiver(_) => true,
        FnArg::Typed(pat_ty) => matches!(&*pat_ty.pat, Pat::Ident(i) if i.ident == "self"),
    }
}

// Pieces of code for the scaffolding args
struct ScaffoldingBits {
    /// Tokenstream that represents the function to call
    ///
    /// For functions, this is simple the function ident.
    /// For methods, this will lift for the `self` param, followed by the method name.
    rust_fn_call: Option<TokenStream>,
    /// Parameters for the scaffolding function
    params: Vec<TokenStream>,
    /// Expressions to lift the arguments in order to pass them to the exported function
    param_lifts: Vec<TokenStream>,
    /// MetadataBuffer calls to build up the metadata
    arg_metadata_calls: Vec<TokenStream>,
}

impl ScaffoldingBits {
    fn new() -> Self {
        Self {
            rust_fn_call: None,
            params: vec![],
            param_lifts: vec![],
            arg_metadata_calls: vec![],
        }
    }

    fn collect_param(
        &mut self,
        param: TokenStream,
        param_lift: TokenStream,
        metadata_builder_call: TokenStream,
    ) {
        self.params.push(param);
        self.param_lifts.push(param_lift);
        self.arg_metadata_calls.push(metadata_builder_call);
    }

    fn collect_param_receiver_error(&mut self, receiver: impl ToTokens, receiver_error_msg: &str) {
        self.collect_param(
            quote! { &self },
            syn::Error::new_spanned(receiver, receiver_error_msg).into_compile_error(),
            quote! {
                .concat_str("<self>")
                .concat(::uniffi::metadata::codes::UNKNOWN)
            },
        );
    }

    fn collect_params<'a>(
        &mut self,
        inputs: impl IntoIterator<Item = &'a FnArg>,
        receiver_error_msg: &'static str,
    ) {
        for (i, arg) in inputs.into_iter().enumerate() {
            let (ty, name) = match arg {
                FnArg::Receiver(r) => {
                    self.collect_param_receiver_error(r, receiver_error_msg);
                    continue;
                }
                FnArg::Typed(pat_ty) => {
                    let name = match &*pat_ty.pat {
                        Pat::Ident(i) if i.ident == "self" => {
                            self.collect_param_receiver_error(i, receiver_error_msg);
                            continue;
                        }
                        Pat::Ident(i) => Some(ident_to_string(&i.ident)),
                        _ => None,
                    };

                    (&pat_ty.ty, name)
                }
            };

            let arg_n = format_ident!("arg{i}");

            // FIXME: With UDL, fallible functions use uniffi::lower_anyhow_error_or_panic instead of
            // panicking unconditionally. This seems cleaner though.
            let panic_fmt = match &name {
                Some(name) => format!("Failed to convert arg '{name}': {{}}"),
                None => format!("Failed to convert arg #{i}: {{}}"),
            };
            let meta_name = name.unwrap_or_else(|| String::from("<missing>"));

            self.collect_param(
                quote! { #arg_n: <#ty as ::uniffi::FfiConverter<crate::UniFfiTag>>::FfiType },
                quote! {
                    <#ty as ::uniffi::FfiConverter<crate::UniFfiTag>>::try_lift(#arg_n)
                        .unwrap_or_else(|err| ::std::panic!(#panic_fmt, err))
                },
                quote! {
                    .concat_str(#meta_name)
                    .concat(<#ty as ::uniffi::FfiConverter<crate::UniFfiTag>>::TYPE_ID_META)
                },
            )
        }
    }

    fn set_rust_fn_call(&mut self, rust_fn_call: TokenStream) {
        self.rust_fn_call = Some(rust_fn_call)
    }

    fn add_self_param(&mut self, param: TokenStream) {
        self.params.insert(0, param)
    }

    fn rust_fn_call(&self) -> TokenStream {
        match &self.rust_fn_call {
            Some(rust_fn_call) => {
                let param_lifts = &self.param_lifts;
                quote! { #rust_fn_call(#(#param_lifts),*) }
            }
            None => panic!("UniFFI Internal error: ScaffoldingBits.func not set"),
        }
    }

    fn gen_function_meta_static_var(
        &self,
        sig: &Signature,
        mod_path: &str,
    ) -> syn::Result<TokenStream> {
        let name = ident_to_string(&sig.ident);
        let return_ty = &sig.output;
        let is_async = sig.is_async;
        let args_len = try_metadata_value_from_usize(
            // Use param_lifts to calculate this instead of sig.inputs to avoid counting any self
            // params
            self.param_lifts.len(),
            "UniFFI limits functions to 256 arguments",
        )?;
        let arg_metadata_calls = &self.arg_metadata_calls;
        Ok(create_metadata_items(
            "func",
            &name,
            quote! {
                ::uniffi::MetadataBuffer::from_code(::uniffi::metadata::codes::FUNC)
                    .concat_str(#mod_path)
                    .concat_str(#name)
                    .concat_bool(#is_async)
                    .concat_value(#args_len)
                    #(#arg_metadata_calls)*
                    .concat(<#return_ty as ::uniffi::FfiConverter<crate::UniFfiTag>>::TYPE_ID_META)
            },
            Some(uniffi_meta::fn_checksum_symbol_name(mod_path, &name)),
        ))
    }

    fn gen_method_meta_static_var(
        &self,
        self_ident: &Ident,
        sig: &Signature,
        mod_path: &str,
    ) -> TokenStream {
        let object_name = ident_to_string(self_ident);
        let name = ident_to_string(&sig.ident);

        let args_len = try_metadata_value_from_usize(
            // Use param_lifts to calculate this instead of sig.inputs to avoid counting any self
            // params
            self.param_lifts.len(),
            "UniFFI limits functions to 256 arguments",
        );
        let metadata_expr = match args_len {
            Ok(args_len) => {
                let return_ty = &sig.output;
                let is_async = sig.is_async;
                let arg_metadata_calls = &self.arg_metadata_calls;

                quote! {
                    ::uniffi::MetadataBuffer::from_code(::uniffi::metadata::codes::METHOD)
                        .concat_str(#mod_path)
                        .concat_str(#object_name)
                        .concat_str(#name)
                        .concat_bool(#is_async)
                        .concat_value(#args_len)
                        #(#arg_metadata_calls)*
                        .concat(
                            <#return_ty as ::uniffi::FfiConverter<crate::UniFfiTag>>::TYPE_ID_META,
                        )
                }
            }
            Err(e) => e.into_compile_error(),
        };

        let symbol_name = uniffi_meta::method_checksum_symbol_name(mod_path, &object_name, &name);
        let name = format!("{object_name}_{name}");
        create_metadata_items("method", &name, metadata_expr, Some(symbol_name))
    }
}

fn gen_ffi_function(
    sig: &Signature,
    ffi_ident: Ident,
    bits: &ScaffoldingBits,
    arguments: &ExportAttributeArguments,
) -> TokenStream {
    let name = ident_to_string(&sig.ident);
    let rust_fn_call = bits.rust_fn_call();
    let fn_params = &bits.params;
    let return_ty = &sig.output;

    if !sig.is_async {
        if let Some(async_runtime) = &arguments.async_runtime {
            return syn::Error::new(
                async_runtime.span(),
                "this attribute is only allowed on async functions",
            )
            .into_compile_error();
        }

        quote! {
            #[doc(hidden)]
            #[no_mangle]
            pub extern "C" fn #ffi_ident(
                #(#fn_params,)*
                call_status: &mut ::uniffi::RustCallStatus,
            ) -> <#return_ty as ::uniffi::FfiConverter<crate::UniFfiTag>>::ReturnType {
                ::uniffi::deps::log::debug!(#name);
                ::uniffi::rust_call(call_status, || {
                    <#return_ty as ::uniffi::FfiConverter<crate::UniFfiTag>>::lower_return(#rust_fn_call)
                })
            }
        }
    } else {
        let rust_future_ctor = match &arguments.async_runtime {
            Some(AsyncRuntime::Tokio(_)) => quote! { new_tokio },
            None => quote! { new },
        };
        let ffi_poll_ident = format_ident!("{ffi_ident}_poll");
        let ffi_drop_ident = format_ident!("{ffi_ident}_drop");

        quote! {
            #[doc(hidden)]
            #[no_mangle]
            pub extern "C" fn #ffi_ident(
                #(#fn_params,)*
                call_status: &mut ::uniffi::RustCallStatus,
            ) -> ::std::boxed::Box<::uniffi::RustFuture<#return_ty>> {
                ::uniffi::deps::log::debug!(#name);
                ::std::boxed::Box::new(::uniffi::RustFuture::#rust_future_ctor(
                    async move { #rust_fn_call.await }
                ))
            }

            // Monomorphised poll function.
            #[doc(hidden)]
            #[no_mangle]
            pub extern "C" fn #ffi_poll_ident(
                future: ::std::option::Option<&mut ::uniffi::RustFuture<#return_ty>>,
                waker: ::std::option::Option<::uniffi::RustFutureForeignWakerFunction>,
                waker_environment: *const ::std::ffi::c_void,
                polled_result: &mut ::std::mem::MaybeUninit<<#return_ty as ::uniffi::FfiConverter<crate::UniFfiTag>>::ReturnType>,
                call_status: &mut ::uniffi::RustCallStatus,
            ) -> ::std::primitive::bool {
                ::uniffi::ffi::uniffi_rustfuture_poll::<_, crate::UniFfiTag>(future, waker, waker_environment, polled_result, call_status)
            }

            // Monomorphised drop function.
            #[doc(hidden)]
            #[no_mangle]
            pub extern "C" fn #ffi_drop_ident(
                future: ::std::option::Option<::std::boxed::Box<::uniffi::RustFuture<#return_ty>>>,
                call_status: &mut ::uniffi::RustCallStatus,
            ) {
                ::uniffi::ffi::uniffi_rustfuture_drop(future, call_status)
            }
        }
    }
}
