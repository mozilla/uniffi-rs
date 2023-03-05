use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Data, DataEnum, DeriveInput, Index, Path, Token, Variant,
};
use uniffi_meta::{ErrorMetadata, VariantMetadata};

use crate::{
    enum_::{enum_ffi_converter_impl, variant_metadata},
    util::{
        assert_type_eq, chain, create_metadata_static_var, either_attribute_arg,
        parse_comma_separated, tagged_impl_header, AttributeSliceExt, UniffiAttribute,
    },
};

pub fn expand_error(input: DeriveInput, module_path: Vec<String>) -> syn::Result<TokenStream> {
    let enum_ = match input.data {
        Data::Enum(e) => e,
        _ => {
            return Err(syn::Error::new(
                Span::call_site(),
                "This derive currently only supports enums",
            ));
        }
    };

    let ident = &input.ident;
    let attr = input.attrs.parse_uniffi_attributes::<ErrorAttr>()?;
    let ffi_converter_impl = error_ffi_converter_impl(ident, &enum_, &attr);

    let meta_static_var = match error_metadata(ident, &enum_.variants, module_path, &attr) {
        Ok(metadata) => create_metadata_static_var(ident, metadata.into()),
        Err(e) => e.into_compile_error(),
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

    Ok(quote! {
        #ffi_converter_impl
        #meta_static_var
        #type_assertion
        #variant_errors
    })
}

pub(crate) fn expand_ffi_converter_error(attr: ErrorAttr, input: DeriveInput) -> TokenStream {
    let enum_ = match input.data {
        Data::Enum(e) => e,
        _ => {
            return syn::Error::new(
                proc_macro2::Span::call_site(),
                "This attribute must only be used on enums",
            )
            .into_compile_error();
        }
    };

    error_ffi_converter_impl(&input.ident, &enum_, &attr)
}

fn error_ffi_converter_impl(ident: &Ident, enum_: &DataEnum, attr: &ErrorAttr) -> TokenStream {
    if attr.flat.is_some() {
        flat_error_ffi_converter_impl(
            ident,
            enum_,
            attr.tag.as_ref(),
            attr.with_try_read.is_some(),
        )
    } else {
        enum_ffi_converter_impl(ident, enum_, attr.tag.as_ref())
    }
}

fn flat_error_ffi_converter_impl(
    ident: &Ident,
    enum_: &DataEnum,
    tag: Option<&Path>,
    implement_try_read: bool,
) -> TokenStream {
    let impl_spec = tagged_impl_header("FfiConverter", ident, tag);

    let write_impl = {
        let match_arms = enum_.variants.iter().enumerate().map(|(i, v)| {
            let v_ident = &v.ident;
            let idx = Index::from(i + 1);

            quote! {
                Self::#v_ident { .. } => {
                    ::uniffi::deps::bytes::BufMut::put_i32(buf, #idx);
                    <::std::string::String as ::uniffi::FfiConverter<()>>::write(error_msg, buf);
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
    syn::custom_keyword!(tag);
    syn::custom_keyword!(flat_error);
    syn::custom_keyword!(with_try_read);
}

#[derive(Default)]
pub(crate) struct ErrorAttr {
    tag: Option<Path>,
    flat: Option<kw::flat_error>,
    with_try_read: Option<kw::with_try_read>,
}

impl UniffiAttribute for ErrorAttr {
    fn parse_one(input: ParseStream<'_>) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::tag) {
            let _: kw::tag = input.parse()?;
            let _: Token![=] = input.parse()?;
            Ok(Self {
                tag: Some(input.parse()?),
                ..Self::default()
            })
        } else if lookahead.peek(kw::flat_error) {
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

    fn merge(self, other: Self) -> syn::Result<Self> {
        Ok(Self {
            tag: either_attribute_arg(self.tag, other.tag)?,
            flat: either_attribute_arg(self.flat, other.flat)?,
            with_try_read: either_attribute_arg(self.with_try_read, other.with_try_read)?,
        })
    }
}

// So ErrorAttr can be used with `parse_macro_input!`
impl Parse for ErrorAttr {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        parse_comma_separated(input)
    }
}
