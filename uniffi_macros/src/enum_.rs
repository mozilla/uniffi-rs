use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{
    parse::ParseStream, spanned::Spanned, Attribute, Data, DataEnum, DeriveInput, Expr, Index, Lit,
    LitStr, Token, Variant,
};

use crate::{
    default::default_value_metadata_calls,
    ffiops,
    record::FieldAttributeArguments,
    util::{
        create_metadata_items, either_attribute_arg, extract_docstring, ident_to_string, kw,
        mod_path, try_metadata_value_from_usize, try_read_field, AttributeSliceExt,
        UniffiAttributeArgs,
    },
    DeriveOptions,
};
use uniffi_meta::EnumShape;

/// Stores parsed data from the Derive Input for the enum.
pub struct EnumItem {
    ident: Ident,
    enum_: DataEnum,
    docstring: String,
    discr_type: Option<Ident>,
    non_exhaustive: bool,
    attr: EnumAttr,
}

impl EnumItem {
    pub fn new(input: DeriveInput) -> syn::Result<Self> {
        let enum_ = match input.data {
            Data::Enum(e) => e,
            _ => {
                return Err(syn::Error::new(
                    Span::call_site(),
                    "This derive must only be used on enums",
                ))
            }
        };
        Ok(Self {
            enum_,
            ident: input.ident,
            docstring: extract_docstring(&input.attrs)?,
            discr_type: Self::extract_repr(&input.attrs)?,
            non_exhaustive: Self::extract_non_exhaustive(&input.attrs),
            attr: input.attrs.parse_uniffi_attr_args()?,
        })
    }

    pub fn extract_repr(attrs: &[Attribute]) -> syn::Result<Option<Ident>> {
        let mut result = None;
        for attr in attrs {
            if attr.path().is_ident("repr") {
                attr.parse_nested_meta(|meta| {
                    result = match meta.path.get_ident() {
                        Some(i) => {
                            let s = i.to_string();
                            match s.as_str() {
                                "u8" | "u16" | "u32" | "u64" | "usize" | "i8" | "i16" | "i32"
                                | "i64" | "isize" => Some(i.clone()),
                                // while the default repr for an enum is `isize` we don't apply that default here.
                                _ => None,
                            }
                        }
                        _ => None,
                    };
                    Ok(())
                })?
            }
        }
        Ok(result)
    }

    pub fn extract_non_exhaustive(attrs: &[Attribute]) -> bool {
        attrs.iter().any(|a| a.path().is_ident("non_exhaustive"))
    }

    pub fn check_attributes_valid_for_enum(&self) -> syn::Result<()> {
        if let Some(flat_error) = &self.attr.flat_error {
            return Err(syn::Error::new(
                flat_error.span(),
                "flat_error not allowed for non-error enums",
            ));
        }
        if let Some(with_try_read) = &self.attr.with_try_read {
            return Err(syn::Error::new(
                with_try_read.span(),
                "with_try_read not allowed for non-error enums",
            ));
        }
        Ok(())
    }

    pub fn ident(&self) -> &Ident {
        &self.ident
    }

    pub fn enum_(&self) -> &DataEnum {
        &self.enum_
    }

    pub fn is_non_exhaustive(&self) -> bool {
        self.non_exhaustive
    }

    pub fn docstring(&self) -> &str {
        self.docstring.as_str()
    }

    pub fn discr_type(&self) -> Option<&Ident> {
        self.discr_type.as_ref()
    }

    pub fn foreign_name(&self) -> String {
        match &self.attr.name {
            Some(name) => name.clone(),
            None => ident_to_string(&self.ident),
        }
    }

    pub fn is_flat_error(&self) -> bool {
        self.attr.flat_error.is_some()
    }

    pub fn generate_error_try_read(&self) -> bool {
        self.attr.with_try_read.is_some()
    }
}

pub fn expand_enum(input: DeriveInput, options: DeriveOptions) -> syn::Result<TokenStream> {
    let item = EnumItem::new(input)?;
    item.check_attributes_valid_for_enum()?;
    let ffi_converter_impl = enum_ffi_converter_impl(&item, &options);

    let meta_static_var = options
        .generate_metadata
        .then(|| enum_meta_static_var(&item).unwrap_or_else(syn::Error::into_compile_error));

    Ok(quote! {
        #ffi_converter_impl
        #meta_static_var
    })
}

pub(crate) fn enum_ffi_converter_impl(item: &EnumItem, options: &DeriveOptions) -> TokenStream {
    enum_or_error_ffi_converter_impl(
        item,
        options,
        quote! { ::uniffi::metadata::codes::TYPE_ENUM },
    )
}

pub(crate) fn rich_error_ffi_converter_impl(
    item: &EnumItem,
    options: &DeriveOptions,
) -> TokenStream {
    enum_or_error_ffi_converter_impl(
        item,
        options,
        quote! { ::uniffi::metadata::codes::TYPE_ENUM },
    )
}

fn enum_or_error_ffi_converter_impl(
    item: &EnumItem,
    options: &DeriveOptions,
    metadata_type_code: TokenStream,
) -> TokenStream {
    let name = &item.foreign_name();
    let ident = item.ident();
    let impl_spec = options.ffi_impl_header("FfiConverter", ident);
    let derive_ffi_traits = options.derive_all_ffi_traits(ident);
    let mod_path = match mod_path() {
        Ok(p) => p,
        Err(e) => return e.into_compile_error(),
    };
    let mut write_match_arms: Vec<_> = item
        .enum_()
        .variants
        .iter()
        .enumerate()
        .map(|(i, v)| {
            let v_ident = &v.ident;
            let field_idents = v
                .fields
                .iter()
                .enumerate()
                .map(|(i, f)| {
                    f.ident
                        .clone()
                        .unwrap_or_else(|| Ident::new(&format!("e{i}"), f.span()))
                })
                .collect::<Vec<Ident>>();
            let idx = Index::from(i + 1);
            let write_fields =
                std::iter::zip(v.fields.iter(), field_idents.iter()).map(|(f, ident)| {
                    let write = ffiops::write(&f.ty);
                    quote! { #write(#ident, buf); }
                });
            let is_tuple = v.fields.iter().any(|f| f.ident.is_none());
            let fields = if is_tuple {
                quote! { ( #(#field_idents),* ) }
            } else {
                quote! { { #(#field_idents),* } }
            };

            quote! {
                Self::#v_ident #fields => {
                    ::uniffi::deps::bytes::BufMut::put_i32(buf, #idx);
                    #(#write_fields)*
                }
            }
        })
        .collect();
    if item.is_non_exhaustive() {
        write_match_arms.push(quote! {
            _ => ::std::panic!("Unexpected variant in non-exhaustive enum"),
        })
    }
    let write_impl = quote! {
        match obj { #(#write_match_arms)* }
    };

    let try_read_match_arms = item.enum_().variants.iter().enumerate().map(|(i, v)| {
        let idx = Index::from(i + 1);
        let v_ident = &v.ident;
        let is_tuple = v.fields.iter().any(|f| f.ident.is_none());
        let try_read_fields = v.fields.iter().map(try_read_field);

        if is_tuple {
            quote! {
                #idx => Self::#v_ident ( #(#try_read_fields)* ),
            }
        } else {
            quote! {
                #idx => Self::#v_ident { #(#try_read_fields)* },
            }
        }
    });
    let error_format_string = format!("Invalid {name} enum value: {{}}");
    let try_read_impl = quote! {
        ::uniffi::check_remaining(buf, 4)?;

        ::std::result::Result::Ok(match ::uniffi::deps::bytes::Buf::get_i32(buf) {
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

pub(crate) fn enum_meta_static_var(item: &EnumItem) -> syn::Result<TokenStream> {
    let name = &item.foreign_name();
    let module_path = mod_path()?;
    let non_exhaustive = item.is_non_exhaustive();
    let docstring = item.docstring();
    let shape = EnumShape::Enum.as_u8();

    let mut metadata_expr = quote! {
        ::uniffi::MetadataBuffer::from_code(::uniffi::metadata::codes::ENUM)
            .concat_str(#module_path)
            .concat_str(#name)
            .concat_value(#shape)
    };
    metadata_expr.extend(match item.discr_type() {
        None => quote! { .concat_bool(false) },
        Some(t) => {
            let type_id_meta = ffiops::type_id_meta(t);
            quote! { .concat_bool(true).concat(#type_id_meta) }
        }
    });
    metadata_expr.extend(variant_metadata(item)?);
    metadata_expr.extend(quote! {
        .concat_bool(#non_exhaustive)
        .concat_long_str(#docstring)
    });
    Ok(create_metadata_items("enum", name, metadata_expr, None))
}

fn variant_value(v: &Variant) -> syn::Result<TokenStream> {
    let Some((_, e)) = &v.discriminant else {
        return Ok(quote! { .concat_bool(false) });
    };
    // Attempting to expose an enum value which we don't understand is a hard-error
    // rather than silently ignoring it. If we had the ability to emit a warning that
    // might make more sense.

    // We can't sanely handle most expressions other than literals, but we can handle
    // negative literals.
    let mut negate = false;
    let lit = match e {
        Expr::Lit(lit) => lit,
        Expr::Unary(expr_unary) if matches!(expr_unary.op, syn::UnOp::Neg(_)) => {
            negate = true;
            match *expr_unary.expr {
                Expr::Lit(ref lit) => lit,
                _ => {
                    return Err(syn::Error::new_spanned(
                        e,
                        "UniFFI disciminant values must be a literal",
                    ));
                }
            }
        }
        _ => {
            return Err(syn::Error::new_spanned(
                e,
                "UniFFI disciminant values must be a literal",
            ));
        }
    };
    let Lit::Int(ref intlit) = lit.lit else {
        return Err(syn::Error::new_spanned(
            v,
            "UniFFI disciminant values must be a literal integer",
        ));
    };
    if !intlit.suffix().is_empty() {
        return Err(syn::Error::new_spanned(
            intlit,
            "integer literals with suffix not supported by UniFFI here",
        ));
    }
    let digits = if negate {
        format!("-{}", intlit.base10_digits())
    } else {
        intlit.base10_digits().to_string()
    };
    Ok(quote! {
        .concat_bool(true)
        .concat_value(::uniffi::metadata::codes::LIT_INT)
        .concat_str(#digits)
    })
}

pub fn variant_metadata(item: &EnumItem) -> syn::Result<Vec<TokenStream>> {
    let enum_ = item.enum_();
    let variants_len =
        try_metadata_value_from_usize(enum_.variants.len(), "UniFFI limits enums to 256 variants")?;
    std::iter::once(Ok(quote! { .concat_value(#variants_len) }))
        .chain(enum_.variants.iter().map(|v| {
            let fields_len = try_metadata_value_from_usize(
                v.fields.len(),
                "UniFFI limits enum variants to 256 fields",
            )?;

            let field_names = v
                .fields
                .iter()
                .map(|f| f.ident.as_ref().map(ident_to_string).unwrap_or_default())
                .collect::<Vec<_>>();

            let field_defaults = v
                .fields
                .iter()
                .map(|f| {
                    let attrs = f
                        .attrs
                        .parse_uniffi_attr_args::<FieldAttributeArguments>()?;
                    default_value_metadata_calls(&attrs.default)
                })
                .collect::<syn::Result<Vec<_>>>()?;
            let name = ident_to_string(&v.ident);
            let value_tokens = variant_value(v)?;

            let docstring = extract_docstring(&v.attrs)?;
            let field_docstrings = v
                .fields
                .iter()
                .map(|f| extract_docstring(&f.attrs))
                .collect::<syn::Result<Vec<_>>>()?;
            let field_type_id_metas = v.fields.iter().map(|f| ffiops::type_id_meta(&f.ty));

            Ok(quote! {
                .concat_str(#name)
                #value_tokens
                .concat_value(#fields_len)
                    #(
                        .concat_str(#field_names)
                        .concat(#field_type_id_metas)
                        #field_defaults
                        .concat_long_str(#field_docstrings)
                    )*
                .concat_long_str(#docstring)
            })
        }))
        .collect()
}

/// Handle #[uniffi(...)] attributes for enums
#[derive(Clone, Default)]
pub struct EnumAttr {
    // All of these attributes are only relevant for errors, but they're defined here so that we
    // can reuse EnumItem for errors.
    pub flat_error: Option<kw::flat_error>,
    pub with_try_read: Option<kw::with_try_read>,
    pub name: Option<String>,
}

impl UniffiAttributeArgs for EnumAttr {
    fn parse_one(input: ParseStream<'_>) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::flat_error) {
            Ok(Self {
                flat_error: input.parse()?,
                ..Self::default()
            })
        } else if lookahead.peek(kw::with_try_read) {
            Ok(Self {
                with_try_read: input.parse()?,
                ..Self::default()
            })
        } else if lookahead.peek(kw::handle_unknown_callback_error) {
            // Not used anymore, but still allowed
            Ok(Self::default())
        } else if lookahead.peek(kw::name) {
            let _: kw::name = input.parse()?;
            let _: Token![=] = input.parse()?;
            let name = Some(input.parse::<LitStr>()?.value());
            Ok(Self {
                name,
                ..Self::default()
            })
        } else {
            Err(lookahead.error())
        }
    }

    fn merge(self, other: Self) -> syn::Result<Self> {
        Ok(Self {
            flat_error: either_attribute_arg(self.flat_error, other.flat_error)?,
            with_try_read: either_attribute_arg(self.with_try_read, other.with_try_read)?,
            name: either_attribute_arg(self.name, other.name)?,
        })
    }
}
