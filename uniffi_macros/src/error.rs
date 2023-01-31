use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    AttributeArgs, Data, DataEnum, DeriveInput, Index, Token, Variant,
};
use uniffi_meta::{ErrorMetadata, VariantMetadata};

use crate::{
    enum_::{enum_ffi_converter_impl, variant_metadata},
    util::{
        assert_type_eq, chain, create_metadata_static_var, either_attribute_arg, AttributeSliceExt,
        FfiConverterTagHandler, UniffiAttribute,
    },
};

pub fn expand_error(input: DeriveInput, module_path: Vec<String>) -> TokenStream {
    let enum_ = match input.data {
        Data::Enum(e) => e,
        _ => {
            return syn::Error::new(
                Span::call_site(),
                "This derive currently only supports enums",
            )
            .into_compile_error()
        }
    };

    let ident = &input.ident;
    let attr = input.attrs.parse_uniffi_attributes::<ErrorAttr>();
    let ffi_converter_impl = match &attr {
        Ok(a) if a.flat.is_some() => flat_error_ffi_converter_impl(
            ident,
            &enum_,
            FfiConverterTagHandler::generic_impl(),
            a.with_try_read.is_some(),
        ),
        _ => enum_ffi_converter_impl(ident, &enum_, FfiConverterTagHandler::generic_impl()),
    };

    let meta_static_var = match &attr {
        Ok(a) => Some(
            match error_metadata(ident, &enum_.variants, module_path, a) {
                Ok(metadata) => create_metadata_static_var(ident, metadata.into()),
                Err(e) => e.into_compile_error(),
            },
        ),
        _ => None,
    };

    let type_assertion = assert_type_eq(ident, quote! { crate::uniffi_types::#ident });
    let variant_errors: TokenStream = enum_
        .variants
        .iter()
        .flat_map(|variant| {
            chain(
                variant.attrs.attributes_not_allowed_here(),
                variant
                    .fields
                    .iter()
                    .flat_map(|field| field.attrs.attributes_not_allowed_here()),
            )
        })
        .map(syn::Error::into_compile_error)
        .collect();
    let attr_error = attr.err().map(syn::Error::into_compile_error);

    quote! {
        #ffi_converter_impl
        #meta_static_var
        #type_assertion
        #variant_errors
        #attr_error
    }
}

pub fn expand_ffi_converter_error(attrs: AttributeArgs, input: DeriveInput) -> TokenStream {
    let tag_handler = match FfiConverterTagHandler::try_from(attrs) {
        Ok(tag_handler) => tag_handler,
        Err(e) => return e.into_compile_error(),
    };
    let enum_ = match input.data {
        Data::Enum(e) => e,
        _ => {
            return syn::Error::new(
                proc_macro2::Span::call_site(),
                "This attribute must only be used on enums",
            )
            .into_compile_error()
        }
    };

    match input.attrs.parse_uniffi_attributes::<ErrorAttr>() {
        Ok(a) if a.flat.is_some() => flat_error_ffi_converter_impl(
            &input.ident,
            &enum_,
            tag_handler,
            a.with_try_read.is_some(),
        ),
        _ => enum_ffi_converter_impl(&input.ident, &enum_, tag_handler),
    }
}

pub(crate) fn flat_error_ffi_converter_impl(
    ident: &Ident,
    enum_: &DataEnum,
    tag_handler: FfiConverterTagHandler,
    implement_try_read: bool,
) -> TokenStream {
    let (impl_spec, tag) = tag_handler.into_impl_and_tag_path("FfiConverter", ident);

    let write_impl = {
        let match_arms = enum_.variants.iter().enumerate().map(|(i, v)| {
            let v_ident = &v.ident;
            let idx = Index::from(i + 1);

            quote! {
                Self::#v_ident { .. } => {
                    ::uniffi::deps::bytes::BufMut::put_i32(buf, #idx);
                    <::std::string::String as ::uniffi::FfiConverter<#tag>>::write(error_msg, buf);
                }
            }
        });

        quote! {
            let error_msg = ::std::string::ToString::to_string(&obj);
            match obj { #(#match_arms)* }
        }
    };

    let try_read_impl = if implement_try_read {
        let match_arms = enum_.variants.iter().enumerate().map(|(i, v)| {
            let v_ident = &v.ident;
            let idx = Index::from(i + 1);

            quote! {
                #idx => Self::#v_ident,
            }
        });
        quote! {
            Ok(match ::uniffi::deps::bytes::Buf::get_i32(buf) {
                #(#match_arms)*
                v => ::uniffi::deps::anyhow::bail!("Invalid #ident enum value: {}", v),
            })
        }
    } else {
        quote! { ::std::panic!("try_read not supported for flat errors") }
    };

    quote! {
        #[automatically_derived]
        unsafe #impl_spec {
            ::uniffi::ffi_converter_rust_buffer_lift_and_lower!(#tag);

            fn write(obj: Self, buf: &mut ::std::vec::Vec<u8>) {
                #write_impl
            }

            fn try_read(buf: &mut &[::std::primitive::u8]) -> ::uniffi::deps::anyhow::Result<Self> {
                #try_read_impl
            }
        }
    }
}

fn error_metadata(
    ident: &Ident,
    variants: &Punctuated<Variant, Token![,]>,
    module_path: Vec<String>,
    attr: &ErrorAttr,
) -> syn::Result<ErrorMetadata> {
    let name = ident.to_string();
    let flat = attr.flat.is_some();
    let variants = if flat {
        variants
            .iter()
            .map(|v| VariantMetadata {
                name: v.ident.to_string(),
                fields: vec![],
            })
            .collect()
    } else {
        variants
            .iter()
            .map(variant_metadata)
            .collect::<syn::Result<_>>()?
    };

    Ok(ErrorMetadata {
        module_path,
        name,
        variants,
        flat,
    })
}

mod kw {
    syn::custom_keyword!(flat_error);
    syn::custom_keyword!(with_try_read);
}

#[derive(Default)]
struct ErrorAttr {
    flat: Option<kw::flat_error>,
    with_try_read: Option<kw::with_try_read>,
}

impl Parse for ErrorAttr {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::flat_error) {
            Ok(Self {
                flat: input.parse()?,
                ..Self::default()
            })
        } else if lookahead.peek(kw::with_try_read) {
            Ok(Self {
                with_try_read: input.parse()?,
                ..Self::default()
            })
        } else {
            Err(lookahead.error())
        }
    }
}

impl UniffiAttribute for ErrorAttr {
    fn merge(self, other: Self) -> syn::Result<Self> {
        Ok(Self {
            flat: either_attribute_arg(self.flat, other.flat)?,
            with_try_read: either_attribute_arg(self.with_try_read, other.with_try_read)?,
        })
    }
}
