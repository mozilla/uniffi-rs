use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{Data, DataStruct, DeriveInput, Field, Fields, Path};
use uniffi_meta::{FieldMetadata, RecordMetadata};

use crate::{
    export::metadata::convert::convert_type,
    util::{
        assert_type_eq, create_metadata_static_var, tagged_impl_header, try_read_field,
        AttributeSliceExt, CommonAttr,
    },
};

pub fn expand_record(input: DeriveInput, module_path: Vec<String>) -> syn::Result<TokenStream> {
    let record = match input.data {
        Data::Struct(s) => s,
        _ => {
            return Err(syn::Error::new(
                Span::call_site(),
                "This derive must only be used on structs",
            ));
        }
    };

    let ident = &input.ident;
    let attr = input.attrs.parse_uniffi_attributes::<CommonAttr>()?;
    let ffi_converter = record_ffi_converter_impl(ident, &record, attr.tag.as_ref());
    let meta_static_var = match record_metadata(ident, record.fields, module_path) {
        Ok(metadata) => create_metadata_static_var(ident, metadata.into()),
        Err(e) => e.into_compile_error(),
    };
    let type_assertion = assert_type_eq(ident, quote! { crate::uniffi_types::#ident });

    Ok(quote! {
        #ffi_converter
        #meta_static_var
        #type_assertion
    })
}

pub(crate) fn expand_record_ffi_converter(attr: CommonAttr, input: DeriveInput) -> TokenStream {
    match input.data {
        Data::Struct(s) => record_ffi_converter_impl(&input.ident, &s, attr.tag.as_ref()),
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
    tag: Option<&Path>,
) -> TokenStream {
    let impl_spec = tagged_impl_header("FfiConverter", ident, tag);
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
