use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{AttributeArgs, Data, DataStruct, DeriveInput, Field, Fields};
use uniffi_meta::{FieldMetadata, RecordMetadata};

use crate::{
    export::metadata::convert::convert_type,
    util::{assert_type_eq, create_metadata_static_var, try_read_field, FfiConverterTagHandler},
};

pub fn expand_record(input: DeriveInput, module_path: Vec<String>) -> TokenStream {
    let record = match input.data {
        Data::Struct(s) => s,
        _ => {
            return syn::Error::new(
                Span::call_site(),
                "This derive must only be used on structs",
            )
            .into_compile_error()
        }
    };

    let ident = &input.ident;
    let ffi_converter =
        record_ffi_converter_impl(ident, &record, FfiConverterTagHandler::generic_impl());
    let meta_static_var = match record_metadata(ident, record.fields, module_path) {
        Ok(metadata) => create_metadata_static_var(ident, metadata.into()),
        Err(e) => e.into_compile_error(),
    };
    let type_assertion = assert_type_eq(ident, quote! { crate::uniffi_types::#ident });

    quote! {
        #ffi_converter
        #meta_static_var
        #type_assertion
    }
}

pub fn expand_record_ffi_converter(attrs: AttributeArgs, input: DeriveInput) -> TokenStream {
    let tag_handler = match FfiConverterTagHandler::try_from(attrs) {
        Ok(tag_handler) => tag_handler,
        Err(e) => return e.into_compile_error(),
    };
    match input.data {
        Data::Struct(s) => record_ffi_converter_impl(&input.ident, &s, tag_handler),
        _ => syn::Error::new(
            proc_macro2::Span::call_site(),
            "This attribute must only be used on structs",
        )
        .into_compile_error(),
    }
}

pub(crate) fn record_ffi_converter_impl(
    ident: &Ident,
    record: &DataStruct,
    tag_handler: FfiConverterTagHandler,
) -> TokenStream {
    let impl_spec = tag_handler.into_impl("FfiConverter", ident);
    let write_impl: TokenStream = record.fields.iter().map(write_field).collect();
    let try_read_fields: TokenStream = record.fields.iter().map(try_read_field).collect();

    quote! {
        #[automatically_derived]
        unsafe #impl_spec {
            ::uniffi::ffi_converter_rust_buffer_lift_and_lower!(crate::UniFfiTag);

            fn write(obj: Self, buf: &mut ::std::vec::Vec<u8>) {
                #write_impl
            }

            fn try_read(buf: &mut &[::std::primitive::u8]) -> ::uniffi::deps::anyhow::Result<Self> {
                Ok(Self { #try_read_fields })
            }
        }
    }
}

fn record_metadata(
    ident: &Ident,
    fields: Fields,
    module_path: Vec<String>,
) -> syn::Result<RecordMetadata> {
    let name = ident.to_string();
    let fields = match fields {
        Fields::Named(fields) => fields.named,
        _ => {
            return Err(syn::Error::new(
                Span::call_site(),
                "UniFFI only supports structs with named fields",
            ));
        }
    };

    let fields = fields
        .iter()
        .map(field_metadata)
        .collect::<syn::Result<_>>()?;

    Ok(RecordMetadata {
        module_path,
        name,
        fields,
    })
}

fn field_metadata(f: &Field) -> syn::Result<FieldMetadata> {
    let name = f.ident.as_ref().unwrap().to_string();

    Ok(FieldMetadata {
        name,
        ty: convert_type(&f.ty)?,
    })
}

fn write_field(f: &Field) -> TokenStream {
    let ident = &f.ident;
    let ty = &f.ty;

    quote! {
        <#ty as ::uniffi::FfiConverter<crate::UniFfiTag>>::write(obj.#ident, buf);
    }
}
