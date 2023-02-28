use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{
    punctuated::Punctuated, AttributeArgs, Data, DataEnum, DeriveInput, Field, Index, Token,
    Variant,
};
use uniffi_meta::{EnumMetadata, FieldMetadata, VariantMetadata};

use crate::{
    export::metadata::convert::convert_type,
    util::{assert_type_eq, create_metadata_static_var, try_read_field, FfiConverterTagHandler},
};

pub fn expand_enum(input: DeriveInput, module_path: Vec<String>) -> TokenStream {
    let enum_ = match input.data {
        Data::Enum(e) => e,
        _ => {
            return syn::Error::new(Span::call_site(), "This derive must only be used on enums")
                .into_compile_error()
        }
    };

    let ident = &input.ident;

    let ffi_converter_impl =
        enum_ffi_converter_impl(ident, &enum_, FfiConverterTagHandler::generic_impl());

    let meta_static_var = {
        match enum_metadata(ident, enum_.variants, module_path) {
            Ok(metadata) => create_metadata_static_var(ident, metadata.into()),
            Err(e) => e.into_compile_error(),
        }
    };

    let type_assertion = assert_type_eq(ident, quote! { crate::uniffi_types::#ident });

    quote! {
        #ffi_converter_impl
        #meta_static_var
        #type_assertion
    }
}

pub fn expand_enum_ffi_converter(attrs: AttributeArgs, input: DeriveInput) -> TokenStream {
    let tag_handler = match FfiConverterTagHandler::try_from(attrs) {
        Ok(tag_handler) => tag_handler,
        Err(e) => return e.into_compile_error(),
    };
    match input.data {
        Data::Enum(e) => enum_ffi_converter_impl(&input.ident, &e, tag_handler),
        _ => syn::Error::new(
            proc_macro2::Span::call_site(),
            "This attribute must only be used on enums",
        )
        .into_compile_error(),
    }
}

pub(crate) fn enum_ffi_converter_impl(
    ident: &Ident,
    enum_: &DataEnum,
    tag_handler: FfiConverterTagHandler,
) -> TokenStream {
    let impl_spec = tag_handler.into_impl("FfiConverter", ident);
    let write_match_arms = enum_.variants.iter().enumerate().map(|(i, v)| {
        let v_ident = &v.ident;
        let fields = v.fields.iter().map(|f| &f.ident);
        let idx = Index::from(i + 1);
        let write_fields = v.fields.iter().map(write_field);

        quote! {
            Self::#v_ident { #(#fields),* } => {
                ::uniffi::deps::bytes::BufMut::put_i32(buf, #idx);
                #(#write_fields)*
            }
        }
    });
    let write_impl = quote! {
        match obj { #(#write_match_arms)* }
    };

    let try_read_match_arms = enum_.variants.iter().enumerate().map(|(i, v)| {
        let idx = Index::from(i + 1);
        let v_ident = &v.ident;
        let try_read_fields = v.fields.iter().map(try_read_field);

        quote! {
            #idx => Self::#v_ident { #(#try_read_fields)* },
        }
    });
    let error_format_string = format!("Invalid {ident} enum value: {{}}");
    let try_read_impl = quote! {
        ::uniffi::check_remaining(buf, 4)?;

        Ok(match ::uniffi::deps::bytes::Buf::get_i32(buf) {
            #(#try_read_match_arms)*
            v => ::uniffi::deps::anyhow::bail!(#error_format_string, v),
        })
    };

    quote! {
        #[automatically_derived]
        unsafe #impl_spec {
            ::uniffi::ffi_converter_rust_buffer_lift_and_lower!(crate::UniFfiTag);

            fn write(obj: Self, buf: &mut ::std::vec::Vec<u8>) {
                #write_impl
            }

            fn try_read(buf: &mut &[::std::primitive::u8]) -> ::uniffi::deps::anyhow::Result<Self> {
                #try_read_impl
            }
        }
    }
}

fn enum_metadata(
    ident: &Ident,
    variants: Punctuated<Variant, Token![,]>,
    module_path: Vec<String>,
) -> syn::Result<EnumMetadata> {
    let name = ident.to_string();
    let variants = variants
        .iter()
        .map(variant_metadata)
        .collect::<syn::Result<_>>()?;

    Ok(EnumMetadata {
        module_path,
        name,
        variants,
    })
}

pub(crate) fn variant_metadata(v: &Variant) -> syn::Result<VariantMetadata> {
    let name = v.ident.to_string();
    let fields = v
        .fields
        .iter()
        .map(|f| field_metadata(f, v))
        .collect::<syn::Result<_>>()?;

    Ok(VariantMetadata { name, fields })
}

fn field_metadata(f: &Field, v: &Variant) -> syn::Result<FieldMetadata> {
    let name = f
        .ident
        .as_ref()
        .ok_or_else(|| {
            syn::Error::new_spanned(
                v,
                "UniFFI only supports enum variants with named fields (or no fields at all)",
            )
        })?
        .to_string();

    Ok(FieldMetadata {
        name,
        ty: convert_type(&f.ty)?,
    })
}

fn write_field(f: &Field) -> TokenStream {
    let ident = &f.ident;
    let ty = &f.ty;

    quote! {
        <#ty as ::uniffi::FfiConverter<crate::UniFfiTag>>::write(#ident, buf);
    }
}
