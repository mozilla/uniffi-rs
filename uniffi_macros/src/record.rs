use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    Data, DataStruct, DeriveInput, Field, Lit, Path, Token,
};

use crate::util::{
    create_metadata_items, either_attribute_arg, ident_to_string, mod_path, tagged_impl_header,
    try_metadata_value_from_usize, try_read_field, ArgumentNotAllowedHere, AttributeSliceExt,
    CommonAttr, UniffiAttributeArgs,
};

pub fn expand_record(input: DeriveInput) -> TokenStream {
    let record = match input.data {
        Data::Struct(s) => s,
        _ => {
            return syn::Error::new(
                Span::call_site(),
                "This derive must only be used on structs",
            )
            .into_compile_error();
        }
    };

    let ident = &input.ident;
    let attr_error = input
        .attrs
        .parse_uniffi_attr_args::<ArgumentNotAllowedHere>()
        .err()
        .map(syn::Error::into_compile_error);
    let ffi_converter = record_ffi_converter_impl(ident, &record, None)
        .unwrap_or_else(syn::Error::into_compile_error);
    let meta_static_var =
        record_meta_static_var(ident, &record).unwrap_or_else(syn::Error::into_compile_error);

    quote! {
        #attr_error
        #ffi_converter
        #meta_static_var
    }
}

pub(crate) fn expand_record_ffi_converter(attr: CommonAttr, input: DeriveInput) -> TokenStream {
    match input.data {
        Data::Struct(s) => record_ffi_converter_impl(&input.ident, &s, attr.tag.as_ref())
            .unwrap_or_else(syn::Error::into_compile_error),
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
) -> syn::Result<TokenStream> {
    let impl_spec = tagged_impl_header("FfiConverter", ident, tag);
    let name = ident_to_string(ident);
    let mod_path = mod_path()?;
    let write_impl: TokenStream = record.fields.iter().map(write_field).collect();
    let try_read_fields: TokenStream = record.fields.iter().map(try_read_field).collect();

    Ok(quote! {
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
                .concat_str(#mod_path)
                .concat_str(#name);
        }
    })
}

fn write_field(f: &Field) -> TokenStream {
    let ident = &f.ident;
    let ty = &f.ty;

    quote! {
        <#ty as ::uniffi::FfiConverter<crate::UniFfiTag>>::write(obj.#ident, buf);
    }
}

mod kw {
    syn::custom_keyword!(default);
    syn::custom_keyword!(None);
}

pub enum FieldDefault {
    Literal(Lit),
    Null(kw::None),
}

impl ToTokens for FieldDefault {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            FieldDefault::Literal(lit) => lit.to_tokens(tokens),
            FieldDefault::Null(kw) => kw.to_tokens(tokens),
        }
    }
}

impl Parse for FieldDefault {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::None) {
            let none_kw: kw::None = input.parse()?;
            Ok(Self::Null(none_kw))
        } else {
            Ok(Self::Literal(input.parse()?))
        }
    }
}

#[derive(Default)]
pub struct FieldAttributeArguments {
    pub(crate) default: Option<FieldDefault>,
}

impl UniffiAttributeArgs for FieldAttributeArguments {
    fn parse_one(input: ParseStream<'_>) -> syn::Result<Self> {
        let _: kw::default = input.parse()?;
        let _: Token![=] = input.parse()?;
        let default = input.parse()?;
        Ok(Self {
            default: Some(default),
        })
    }

    fn merge(self, other: Self) -> syn::Result<Self> {
        Ok(Self {
            default: either_attribute_arg(self.default, other.default)?,
        })
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

    let concat_fields: TokenStream = record
        .fields
        .iter()
        .map(|f| {
            let attrs = f
                .attrs
                .parse_uniffi_attr_args::<FieldAttributeArguments>()?;

            let name = ident_to_string(f.ident.as_ref().unwrap());
            let ty = &f.ty;
            let default = match attrs.default {
                Some(default) => {
                    let default_value = default_value_concat_calls(default)?;
                    quote! {
                        .concat_bool(true)
                        #default_value
                    }
                }
                None => quote! { .concat_bool(false) },
            };

            Ok(quote! {
                .concat_str(#name)
                .concat(<#ty as ::uniffi::FfiConverter<crate::UniFfiTag>>::TYPE_ID_META)
                #default
            })
        })
        .collect::<syn::Result<_>>()?;

    Ok(create_metadata_items(
        "record",
        &name,
        quote! {
            ::uniffi::MetadataBuffer::from_code(::uniffi::metadata::codes::RECORD)
                .concat_str(#module_path)
                .concat_str(#name)
                .concat_value(#fields_len)
                #concat_fields
        },
        None,
    ))
}

fn default_value_concat_calls(default: FieldDefault) -> syn::Result<TokenStream> {
    match default {
        FieldDefault::Literal(Lit::Int(i)) if !i.suffix().is_empty() => Err(
            syn::Error::new_spanned(i, "integer literals with suffix not supported here"),
        ),
        FieldDefault::Literal(Lit::Float(f)) if !f.suffix().is_empty() => Err(
            syn::Error::new_spanned(f, "float literals with suffix not supported here"),
        ),

        FieldDefault::Literal(Lit::Str(s)) => Ok(quote! {
            .concat_value(::uniffi::metadata::codes::LIT_STR)
            .concat_str(#s)
        }),
        FieldDefault::Literal(Lit::Int(i)) => {
            let digits = i.base10_digits();
            Ok(quote! {
                .concat_value(::uniffi::metadata::codes::LIT_INT)
                .concat_str(#digits)
            })
        }
        FieldDefault::Literal(Lit::Float(f)) => {
            let digits = f.base10_digits();
            Ok(quote! {
                .concat_value(::uniffi::metadata::codes::LIT_FLOAT)
                .concat_str(#digits)
            })
        }
        FieldDefault::Literal(Lit::Bool(b)) => Ok(quote! {
            .concat_value(::uniffi::metadata::codes::LIT_BOOL)
            .concat_bool(#b)
        }),

        FieldDefault::Literal(_) => Err(syn::Error::new_spanned(
            default,
            "this type of literal is not currently supported as a default",
        )),

        FieldDefault::Null(_) => Ok(quote! {
            .concat_value(::uniffi::metadata::codes::LIT_NULL)
        }),
    }
}
