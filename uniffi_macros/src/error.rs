use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{punctuated::Punctuated, Data, DeriveInput, Token, Variant};
use uniffi_meta::ErrorMetadata;

use crate::{
    enum_::{enum_ffi_converter_impl, variant_metadata},
    util::{assert_type_eq, create_metadata_static_var},
};

pub fn expand_error(input: DeriveInput, module_path: Vec<String>) -> TokenStream {
    let variants = match input.data {
        Data::Enum(e) => Some(e.variants),
        _ => None,
    };

    let ident = &input.ident;

    let ffi_converter_impl = enum_ffi_converter_impl(&variants, ident);

    let meta_static_var = if let Some(variants) = variants {
        match error_metadata(ident, variants, module_path) {
            Ok(metadata) => create_metadata_static_var(ident, metadata.into()),
            Err(e) => e.into_compile_error(),
        }
    } else {
        syn::Error::new(
            Span::call_site(),
            "This derive currently only supports enums",
        )
        .into_compile_error()
    };

    let type_assertion = assert_type_eq(ident, quote! { crate::uniffi_types::#ident });

    quote! {
        #ffi_converter_impl

        #[automatically_derived]
        impl ::uniffi::FfiError for #ident {}

        #meta_static_var
        #type_assertion
    }
}

fn error_metadata(
    ident: &Ident,
    variants: Punctuated<Variant, Token![,]>,
    module_path: Vec<String>,
) -> syn::Result<ErrorMetadata> {
    let name = ident.to_string();
    let variants = variants
        .iter()
        .map(variant_metadata)
        .collect::<syn::Result<_>>()?;

    Ok(ErrorMetadata {
        module_path,
        name,
        variants,
    })
}
