use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    Data, DataEnum, DeriveInput, Expr, Field, Index, Lit, Variant,
};

use crate::util::{
    create_metadata_items, derive_all_ffi_traits, either_attribute_arg, extract_docstring,
    ident_to_string, kw, mod_path, parse_comma_separated, tagged_impl_header,
    try_metadata_value_from_usize, try_read_field, AttributeSliceExt, UniffiAttributeArgs,
};

pub fn expand_enum(
    input: DeriveInput,
    // Attributes from #[derive_error_for_udl()], if we are in udl mode
    attr_from_udl_mode: Option<EnumAttr>,
    udl_mode: bool,
) -> syn::Result<TokenStream> {
    let enum_ = match input.data {
        Data::Enum(e) => e,
        _ => {
            return Err(syn::Error::new(
                Span::call_site(),
                "This derive must only be used on enums",
            ))
        }
    };
    let ident = &input.ident;
    let docstring = extract_docstring(&input.attrs)?;
    let mut attr: EnumAttr = input.attrs.parse_uniffi_attr_args()?;
    if let Some(attr_from_udl_mode) = attr_from_udl_mode {
        attr = attr.merge(attr_from_udl_mode)?;
    }
    let ffi_converter_impl = enum_ffi_converter_impl(ident, &enum_, udl_mode, &attr);

    let meta_static_var = (!udl_mode).then(|| {
        enum_meta_static_var(ident, docstring, &enum_, &attr)
            .unwrap_or_else(syn::Error::into_compile_error)
    });

    Ok(quote! {
        #ffi_converter_impl
        #meta_static_var
    })
}

pub(crate) fn enum_ffi_converter_impl(
    ident: &Ident,
    enum_: &DataEnum,
    udl_mode: bool,
    attr: &EnumAttr,
) -> TokenStream {
    enum_or_error_ffi_converter_impl(
        ident,
        enum_,
        udl_mode,
        attr,
        quote! { ::uniffi::metadata::codes::TYPE_ENUM },
    )
}

pub(crate) fn rich_error_ffi_converter_impl(
    ident: &Ident,
    enum_: &DataEnum,
    udl_mode: bool,
    attr: &EnumAttr,
) -> TokenStream {
    enum_or_error_ffi_converter_impl(
        ident,
        enum_,
        udl_mode,
        attr,
        quote! { ::uniffi::metadata::codes::TYPE_ENUM },
    )
}

fn enum_or_error_ffi_converter_impl(
    ident: &Ident,
    enum_: &DataEnum,
    udl_mode: bool,
    attr: &EnumAttr,
    metadata_type_code: TokenStream,
) -> TokenStream {
    let name = ident_to_string(ident);
    let impl_spec = tagged_impl_header("FfiConverter", ident, udl_mode);
    let derive_ffi_traits = derive_all_ffi_traits(ident, udl_mode);
    let mod_path = match mod_path() {
        Ok(p) => p,
        Err(e) => return e.into_compile_error(),
    };
    let mut write_match_arms: Vec<_> = enum_
        .variants
        .iter()
        .enumerate()
        .map(|(i, v)| {
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
        })
        .collect();
    if attr.non_exhaustive.is_some() {
        write_match_arms.push(quote! {
            _ => panic!("Unexpected variant in non-exhaustive enum"),
        })
    }
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

            const TYPE_ID_META: ::uniffi::MetadataBuffer = ::uniffi::MetadataBuffer::from_code(#metadata_type_code)
                .concat_str(#mod_path)
                .concat_str(#name);
        }

        #derive_ffi_traits
    }
}

fn write_field(f: &Field) -> TokenStream {
    let ident = &f.ident;
    let ty = &f.ty;

    quote! {
        <#ty as ::uniffi::Lower<crate::UniFfiTag>>::write(#ident, buf);
    }
}

pub(crate) fn enum_meta_static_var(
    ident: &Ident,
    docstring: String,
    enum_: &DataEnum,
    attr: &EnumAttr,
) -> syn::Result<TokenStream> {
    let name = ident_to_string(ident);
    let module_path = mod_path()?;
    let non_exhaustive = attr.non_exhaustive.is_some();

    let mut metadata_expr = quote! {
        ::uniffi::MetadataBuffer::from_code(::uniffi::metadata::codes::ENUM)
            .concat_str(#module_path)
            .concat_str(#name)
    };
    metadata_expr.extend(variant_metadata(enum_)?);
    metadata_expr.extend(quote! {
        .concat_bool(#non_exhaustive)
        .concat_long_str(#docstring)
    });
    Ok(create_metadata_items("enum", &name, metadata_expr, None))
}

fn variant_value(v: &Variant) -> syn::Result<TokenStream> {
    let Some((_, e)) = &v.discriminant else {
        return Ok(quote! { .concat_bool(false) });
    };
    let Expr::Lit(lit) = e else {
        return Ok(quote! { .concat_bool(false) });
    };
    let Lit::Int(ref intlit) = lit.lit else {
        return Ok(quote! { .concat_bool(false) });
    };
    if !intlit.suffix().is_empty() {
        return Err(syn::Error::new_spanned(
            intlit,
            "integer literals with suffix not supported here",
        ));
    }
    let digits = intlit.base10_digits();
    Ok(quote! {
        .concat_bool(true)
        .concat_value(::uniffi::metadata::codes::LIT_INT)
        .concat_str(#digits)
    })
}

pub fn variant_metadata(enum_: &DataEnum) -> syn::Result<Vec<TokenStream>> {
    let variants_len =
        try_metadata_value_from_usize(enum_.variants.len(), "UniFFI limits enums to 256 variants")?;
    std::iter::once(Ok(quote! { .concat_value(#variants_len) }))
        .chain(
            enum_.variants
                .iter()
                .map(|v| {
                    let fields_len = try_metadata_value_from_usize(
                        v.fields.len(),
                        "UniFFI limits enum variants to 256 fields",
                    )?;

                    let field_names = v.fields
                        .iter()
                        .map(|f| {
                            f.ident
                                .as_ref()
                                .ok_or_else(||
                                    syn::Error::new_spanned(
                                        v,
                                        "UniFFI only supports enum variants with named fields (or no fields at all)",
                                    )
                                )
                                .map(ident_to_string)
                        })
                    .collect::<syn::Result<Vec<_>>>()?;

                    let name = ident_to_string(&v.ident);
                    let value_tokens = variant_value(v)?;
                    let docstring = extract_docstring(&v.attrs)?;
                    let field_types = v.fields.iter().map(|f| &f.ty);
                    let field_docstrings = v.fields
                        .iter()
                        .map(|f| extract_docstring(&f.attrs))
                        .collect::<syn::Result<Vec<_>>>()?;

                    Ok(quote! {
                        .concat_str(#name)
                        #value_tokens
                        .concat_value(#fields_len)
                            #(
                                .concat_str(#field_names)
                                .concat(<#field_types as ::uniffi::Lower<crate::UniFfiTag>>::TYPE_ID_META)
                                // field defaults not yet supported for enums
                                .concat_bool(false)
                                .concat_long_str(#field_docstrings)
                            )*
                        .concat_long_str(#docstring)
                    })
                })
        )
        .collect()
}

#[derive(Default)]
pub struct EnumAttr {
    pub non_exhaustive: Option<kw::non_exhaustive>,
}

// So ErrorAttr can be used with `parse_macro_input!`
impl Parse for EnumAttr {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        parse_comma_separated(input)
    }
}

impl UniffiAttributeArgs for EnumAttr {
    fn parse_one(input: ParseStream<'_>) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::non_exhaustive) {
            Ok(Self {
                non_exhaustive: input.parse()?,
            })
        } else {
            Err(lookahead.error())
        }
    }

    fn merge(self, other: Self) -> syn::Result<Self> {
        Ok(Self {
            non_exhaustive: either_attribute_arg(self.non_exhaustive, other.non_exhaustive)?,
        })
    }
}
