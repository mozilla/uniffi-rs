use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    Data, DataEnum, DeriveInput, Index,
};

use crate::{
    enum_::{handle_callback_unexpected_error_fn, rich_error_ffi_converter_impl, variant_metadata},
    util::{
        chain, create_metadata_items, either_attribute_arg, ident_to_string, kw, mod_path,
        parse_comma_separated, tagged_impl_header, try_metadata_value_from_usize,
        AttributeSliceExt, UniffiAttributeArgs,
    },
};

pub fn expand_error(
    input: DeriveInput,
    // Attributes from #[derive_error_for_udl()], if we are in udl mode
    attr_from_udl_mode: Option<ErrorAttr>,
    udl_mode: bool,
) -> syn::Result<TokenStream> {
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
    let mut attr: ErrorAttr = input.attrs.parse_uniffi_attr_args()?;
    if let Some(attr_from_udl_mode) = attr_from_udl_mode {
        attr = attr.merge(attr_from_udl_mode)?;
    }
    let ffi_converter_impl = error_ffi_converter_impl(ident, &enum_, &attr, udl_mode);
    let meta_static_var = (!udl_mode).then(|| {
        error_meta_static_var(ident, &enum_, attr.flat.is_some())
            .unwrap_or_else(syn::Error::into_compile_error)
    });

    let variant_errors: TokenStream = enum_
        .variants
        .iter()
        .flat_map(|variant| {
            chain(
                variant.attrs.uniffi_attr_args_not_allowed_here(),
                variant
                    .fields
                    .iter()
                    .flat_map(|field| field.attrs.uniffi_attr_args_not_allowed_here()),
            )
        })
        .map(syn::Error::into_compile_error)
        .collect();

    Ok(quote! {
        #ffi_converter_impl
        #meta_static_var
        #variant_errors
    })
}

fn error_ffi_converter_impl(
    ident: &Ident,
    enum_: &DataEnum,
    attr: &ErrorAttr,
    udl_mode: bool,
) -> TokenStream {
    if attr.flat.is_some() {
        flat_error_ffi_converter_impl(
            ident,
            enum_,
            udl_mode,
            attr.with_try_read.is_some(),
            attr.handle_unknown_callback_error.is_some(),
        )
    } else {
        rich_error_ffi_converter_impl(
            ident,
            enum_,
            udl_mode,
            attr.handle_unknown_callback_error.is_some(),
        )
    }
}

// FfiConverters for "flat errors"
//
// These are errors where we only lower the to_string() value, rather than any assocated data.
// We lower the to_string() value unconditionally, whether the enum has associated data or not.
fn flat_error_ffi_converter_impl(
    ident: &Ident,
    enum_: &DataEnum,
    udl_mode: bool,
    implement_try_read: bool,
    handle_unknown_callback_error: bool,
) -> TokenStream {
    let name = ident_to_string(ident);
    let impl_spec = tagged_impl_header("FfiConverter", ident, udl_mode);
    let lift_ref_impl_spec = tagged_impl_header("LiftRef", ident, udl_mode);
    let mod_path = match mod_path() {
        Ok(p) => p,
        Err(e) => return e.into_compile_error(),
    };

    let write_impl = {
        let match_arms = enum_.variants.iter().enumerate().map(|(i, v)| {
            let v_ident = &v.ident;
            let idx = Index::from(i + 1);

            quote! {
                Self::#v_ident { .. } => {
                    ::uniffi::deps::bytes::BufMut::put_i32(buf, #idx);
                    <::std::string::String as ::uniffi::FfiConverter<crate::UniFfiTag>>::write(error_msg, buf);
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

    let handle_callback_unexpected_error =
        handle_callback_unexpected_error_fn(handle_unknown_callback_error);

    quote! {
        #[automatically_derived]
        unsafe #impl_spec {
            ::uniffi::ffi_converter_rust_buffer_lift_and_lower!(crate::UniFfiTag);
            ::uniffi::ffi_converter_default_return!(crate::UniFfiTag);

            fn write(obj: Self, buf: &mut ::std::vec::Vec<u8>) {
                #write_impl
            }

            fn try_read(buf: &mut &[::std::primitive::u8]) -> ::uniffi::deps::anyhow::Result<Self> {
                #try_read_impl
            }

            #handle_callback_unexpected_error

            const TYPE_ID_META: ::uniffi::MetadataBuffer = ::uniffi::MetadataBuffer::from_code(::uniffi::metadata::codes::TYPE_ENUM)
                .concat_str(#mod_path)
                .concat_str(#name);
        }

        #lift_ref_impl_spec {
            type LiftType = Self;
        }
    }
}

pub(crate) fn error_meta_static_var(
    ident: &Ident,
    enum_: &DataEnum,
    flat: bool,
) -> syn::Result<TokenStream> {
    let name = ident_to_string(ident);
    let module_path = mod_path()?;
    let mut metadata_expr = quote! {
            ::uniffi::MetadataBuffer::from_code(::uniffi::metadata::codes::ERROR)
                // first our is-flat flag
                .concat_bool(#flat)
                // followed by an enum
                .concat_str(#module_path)
                .concat_str(#name)
    };
    if flat {
        metadata_expr.extend(flat_error_variant_metadata(enum_)?)
    } else {
        metadata_expr.extend(variant_metadata(enum_)?);
    }
    Ok(create_metadata_items("error", &name, metadata_expr, None))
}

pub fn flat_error_variant_metadata(enum_: &DataEnum) -> syn::Result<Vec<TokenStream>> {
    let variants_len =
        try_metadata_value_from_usize(enum_.variants.len(), "UniFFI limits enums to 256 variants")?;
    Ok(std::iter::once(quote! { .concat_value(#variants_len) })
        .chain(enum_.variants.iter().map(|v| {
            let name = ident_to_string(&v.ident);
            quote! { .concat_str(#name) }
        }))
        .collect())
}

#[derive(Default)]
pub struct ErrorAttr {
    flat: Option<kw::flat_error>,
    with_try_read: Option<kw::with_try_read>,
    /// Can this error be used in a callback interface?
    handle_unknown_callback_error: Option<kw::handle_unknown_callback_error>,
}

impl UniffiAttributeArgs for ErrorAttr {
    fn parse_one(input: ParseStream<'_>) -> syn::Result<Self> {
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
        } else if lookahead.peek(kw::handle_unknown_callback_error) {
            Ok(Self {
                handle_unknown_callback_error: input.parse()?,
                ..Self::default()
            })
        } else {
            Err(lookahead.error())
        }
    }

    fn merge(self, other: Self) -> syn::Result<Self> {
        Ok(Self {
            flat: either_attribute_arg(self.flat, other.flat)?,
            with_try_read: either_attribute_arg(self.with_try_read, other.with_try_read)?,
            handle_unknown_callback_error: either_attribute_arg(
                self.handle_unknown_callback_error,
                other.handle_unknown_callback_error,
            )?,
        })
    }
}

// So ErrorAttr can be used with `parse_macro_input!`
impl Parse for ErrorAttr {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        parse_comma_separated(input)
    }
}
