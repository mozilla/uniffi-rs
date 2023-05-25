use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{Data, DataStruct, DeriveInput, Field, Path};

use crate::util::{
    create_metadata_items, ident_to_string, mod_path, tagged_impl_header,
    try_metadata_value_from_usize, try_read_field, AttributeSliceExt, CommonAttr,
};

pub fn expand_record(input: DeriveInput) -> syn::Result<TokenStream> {
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
    let attr = input.attrs.parse_uniffi_attr_args::<CommonAttr>()?;
    let ffi_converter = record_ffi_converter_impl(ident, &record, attr.tag.as_ref());
    let meta_static_var = record_meta_static_var(ident, &record)?;

    Ok(quote! {
        #ffi_converter
        #meta_static_var
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
    let name = ident_to_string(ident);
    let write_impl: TokenStream = record.fields.iter().map(write_field).collect();
    let try_read_fields: TokenStream = record.fields.iter().map(try_read_field).collect();

    quote! {
        #[automatically_derived]
        unsafe #impl_spec {
            ::uniffi::ffi_converter_rust_buffer_lift_and_lower!(crate::UniFfiTag);
            ::uniffi::ffi_converter_default_return!(crate::UniFfiTag);

            fn write(obj: Self, buf: &mut ::std::vec::Vec<u8>) {
                #write_impl
            }

            fn try_read(buf: &mut &[::std::primitive::u8]) -> ::uniffi::deps::anyhow::Result<Self> {
                Ok(Self { #try_read_fields })
            }

            const TYPE_ID_META: ::uniffi::MetadataBuffer = ::uniffi::MetadataBuffer::from_code(::uniffi::metadata::codes::TYPE_RECORD)
                .concat_str(#name);
        }
    }
}

fn write_field(f: &Field) -> TokenStream {
    let ident = &f.ident;
    let ty = &f.ty;

    quote! {
        <#ty as ::uniffi::FfiConverter<crate::UniFfiTag>>::write(obj.#ident, buf);
    }
}

pub(crate) fn record_meta_static_var(
    ident: &Ident,
    record: &DataStruct,
) -> syn::Result<TokenStream> {
    let name = ident_to_string(ident);
    let module_path = mod_path()?;
    let fields_len =
        try_metadata_value_from_usize(record.fields.len(), "UniFFI limits structs to 256 fields")?;

    let concat_fields = record.fields.iter().map(|f| {
        let name = ident_to_string(f.ident.as_ref().unwrap());
        let ty = &f.ty;
        quote! {
            .concat_str(#name)
            .concat(<#ty as ::uniffi::FfiConverter<crate::UniFfiTag>>::TYPE_ID_META)
        }
    });

    Ok(create_metadata_items(
        "record",
        &name,
        quote! {
            ::uniffi::MetadataBuffer::from_code(::uniffi::metadata::codes::RECORD)
                .concat_str(#module_path)
                .concat_str(#name)
                .concat_value(#fields_len)
                #(#concat_fields)*
        },
        None,
    ))
}
