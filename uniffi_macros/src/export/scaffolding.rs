use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{FnArg, ItemFn, Pat, ReturnType};

use super::ExportItem;

pub(super) fn gen_scaffolding(item: ExportItem, mod_path: &[String]) -> syn::Result<TokenStream> {
    match item {
        ExportItem::Function {
            item,
            checksum,
            tracked_file,
        } => {
            let scaffolding = gen_fn_scaffolding(&item, mod_path, checksum)?;
            Ok(quote! {
                #scaffolding
                #tracked_file
            })
        }
    }
}

fn gen_fn_scaffolding(
    item: &ItemFn,
    mod_path: &[String],
    checksum: u16,
) -> syn::Result<TokenStream> {
    let name = &item.sig.ident;
    let name_s = name.to_string();
    let ffi_name = format_ident!("__uniffi_{}_{}_{:x}", mod_path.join("__"), name, checksum);

    let mut params = Vec::new();
    let mut args = Vec::new();

    for (i, arg) in item.sig.inputs.iter().enumerate() {
        match arg {
            FnArg::Receiver(receiver) => {
                return Err(syn::Error::new_spanned(
                    receiver,
                    "methods are not yet supported by uniffi::export",
                ));
            }
            FnArg::Typed(pat_ty) => {
                let ty = &pat_ty.ty;
                let name = format_ident!("arg{}", i);

                params.push(quote! { #name: <#ty as ::uniffi::FfiConverter>::FfiType });

                let panic_fmt = match &*pat_ty.pat {
                    Pat::Ident(i) => {
                        format!("Failed to convert arg '{}': {{}}", i.ident)
                    }
                    _ => {
                        format!("Failed to convert arg #{}: {{}}", i)
                    }
                };
                args.push(quote! {
                    <#ty as ::uniffi::FfiConverter>::try_lift(#name).unwrap_or_else(|err| {
                        ::std::panic!(#panic_fmt, err)
                    })
                });
            }
        }
    }

    let fn_call = quote! {
        #name(#(#args),*)
    };

    // FIXME(jplatte): Use an extra trait implemented for `T: FfiConverter` as
    // well as `()` so no different codegen is needed?
    let (output, return_expr);
    match &item.sig.output {
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

    Ok(quote! {
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
    })
}
