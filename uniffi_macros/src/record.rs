use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Data, DeriveInput, Fields};
use uniffi_meta::{FieldMetadata, RecordMetadata};

use crate::{
    export::metadata::convert::convert_type,
    util::{assert_type_eq, create_metadata_static_var, try_read_field},
};

pub fn expand_record(input: DeriveInput, module_path: Vec<String>) -> TokenStream {
    let fields = match input.data {
        Data::Struct(s) => Some(s.fields),
        _ => None,
    };

    let ident = &input.ident;

    let (write_impl, try_read_fields) = match &fields {
        Some(fields) => (
            fields.iter().map(write_field).collect(),
            fields.iter().map(try_read_field).collect(),
        ),
        None => {
            let unimplemented = quote! { ::std::unimplemented!() };
            (unimplemented.clone(), unimplemented)
        }
    };

    let meta_static_var = fields
        .map(|fields| {
            let name = ident.to_string();
            let fields = match fields {
                Fields::Named(fields) => fields.named,
                _ => {
                    return syn::Error::new(
                        Span::call_site(),
                        "UniFFI only supports structs with named fields",
                    )
                    .into_compile_error();
                }
            };

            let fields_res: syn::Result<_> = fields
                .iter()
                .map(|f| {
                    let name = f.ident.as_ref().unwrap().to_string();

                    Ok(FieldMetadata {
                        name,
                        ty: convert_type(&f.ty)?,
                    })
                })
                .collect();

            match fields_res {
                Ok(fields) => {
                    let metadata = RecordMetadata {
                        module_path,
                        name,
                        fields,
                    };

                    create_metadata_static_var(ident, metadata.into())
                }
                Err(e) => e.into_compile_error(),
            }
        })
        .unwrap_or_else(|| {
            syn::Error::new(
                Span::call_site(),
                "This derive must only be used on structs",
            )
            .into_compile_error()
        });

    let type_assertion = assert_type_eq(ident, quote! { crate::uniffi_types::#ident });

    quote! {
        impl ::uniffi::RustBufferFfiConverter for #ident {
            type RustType = Self;

            fn write(obj: Self, buf: &mut ::std::vec::Vec<u8>) {
                #write_impl
            }

            fn try_read(buf: &mut &[::std::primitive::u8]) -> ::uniffi::deps::anyhow::Result<Self> {
                Ok(Self { #try_read_fields })
            }
        }

        #meta_static_var
        #type_assertion
    }
}

pub fn write_field(f: &syn::Field) -> TokenStream {
    let ident = &f.ident;
    let ty = &f.ty;

    quote! {
        <#ty as ::uniffi::FfiConverter>::write(obj.#ident, buf);
    }
}
