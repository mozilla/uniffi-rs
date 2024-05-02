use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, Index};

use crate::{
    enum_::{rich_error_ffi_converter_impl, variant_metadata, EnumItem},
    ffiops,
    util::{
        chain, create_metadata_items, extract_docstring, ident_to_string, mod_path,
        try_metadata_value_from_usize, AttributeSliceExt,
    },
    DeriveOptions,
};

pub fn expand_error(input: DeriveInput, options: DeriveOptions) -> syn::Result<TokenStream> {
    let enum_item = EnumItem::new(input)?;
    let ffi_converter_impl = error_ffi_converter_impl(&enum_item, &options)?;
    let meta_static_var = options
        .generate_metadata
        .then(|| error_meta_static_var(&enum_item).unwrap_or_else(syn::Error::into_compile_error));

    let variant_errors: TokenStream = enum_item
        .enum_()
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

fn error_ffi_converter_impl(item: &EnumItem, options: &DeriveOptions) -> syn::Result<TokenStream> {
    Ok(if item.is_flat_error() {
        flat_error_ffi_converter_impl(item, options)
    } else {
        rich_error_ffi_converter_impl(item, options)
    })
}

// FfiConverters for "flat errors"
//
// These are errors where we only lower the to_string() value, rather than any associated data.
// We lower the to_string() value unconditionally, whether the enum has associated data or not.
fn flat_error_ffi_converter_impl(item: &EnumItem, options: &DeriveOptions) -> TokenStream {
    let name = item.name();
    let ident = item.ident();
    let lower_impl_spec = options.ffi_impl_header("Lower", ident);
    let lift_impl_spec = options.ffi_impl_header("Lift", ident);
    let type_id_impl_spec = options.ffi_impl_header("TypeId", ident);
    let derive_ffi_traits = options.derive_ffi_traits(ident, &["ConvertError"]);
    let mod_path = match mod_path() {
        Ok(p) => p,
        Err(e) => return e.into_compile_error(),
    };

    let lower_impl = {
        let mut match_arms: Vec<_> = item
            .enum_()
            .variants
            .iter()
            .enumerate()
            .map(|(i, v)| {
                let v_ident = &v.ident;
                let idx = Index::from(i + 1);
                let write_string = ffiops::write(quote! { ::std::string::String });

                quote! {
                    Self::#v_ident { .. } => {
                        ::uniffi::deps::bytes::BufMut::put_i32(buf, #idx);
                        #write_string(error_msg, buf);
                    }
                }
            })
            .collect();
        if item.is_non_exhaustive() {
            match_arms.push(quote! {
                _ => panic!("Unexpected variant in non-exhaustive enum"),
            })
        }

        let lower = ffiops::lower_into_rust_buffer(quote! { Self });

        quote! {
            #[automatically_derived]
            unsafe #lower_impl_spec {
                type FfiType = ::uniffi::RustBuffer;

                fn write(obj: Self, buf: &mut ::std::vec::Vec<u8>) {
                    let error_msg = ::std::string::ToString::to_string(&obj);
                    match obj { #(#match_arms)* }
                }

                fn lower(obj: Self) -> ::uniffi::RustBuffer {
                    #lower(obj)
                }
            }
        }
    };

    let lift_impl = if item.generate_error_try_read() {
        let match_arms = item.enum_().variants.iter().enumerate().map(|(i, v)| {
            let v_ident = &v.ident;
            let idx = Index::from(i + 1);

            quote! {
                #idx => Self::#v_ident,
            }
        });
        let try_lift = ffiops::try_lift_from_rust_buffer(quote! { Self });
        quote! {
            #[automatically_derived]
            unsafe #lift_impl_spec {
                type FfiType = ::uniffi::RustBuffer;

                fn try_read(buf: &mut &[::std::primitive::u8]) -> ::uniffi::deps::anyhow::Result<Self> {
                    Ok(match ::uniffi::deps::bytes::Buf::get_i32(buf) {
                        #(#match_arms)*
                        v => ::uniffi::deps::anyhow::bail!("Invalid #ident enum value: {}", v),
                    })
                }

                fn try_lift(v: ::uniffi::RustBuffer) -> ::uniffi::deps::anyhow::Result<Self> {
                    #try_lift(v)
                }
            }
        }
    } else {
        quote! {
            // Lifting flat errors is not currently supported, but we still define the trait so
            // that dicts containing flat errors don't cause compile errors (see ReturnOnlyDict in
            // coverall.rs).
            //
            // Note: it would be better to not derive `Lift` for dictionaries containing flat
            // errors, but getting the trait bounds and derived impls there would be much harder.
            // For now, we just fail at runtime.
            #[automatically_derived]
            unsafe #lift_impl_spec {
                type FfiType = ::uniffi::RustBuffer;

                fn try_read(buf: &mut &[::std::primitive::u8]) -> ::uniffi::deps::anyhow::Result<Self> {
                    panic!("Can't lift flat errors")
                }

                fn try_lift(v: ::uniffi::RustBuffer) -> ::uniffi::deps::anyhow::Result<Self> {
                    panic!("Can't lift flat errors")
                }
            }
        }
    };

    quote! {
        #lower_impl
        #lift_impl

        #[automatically_derived]
        #type_id_impl_spec {
            const TYPE_ID_META: ::uniffi::MetadataBuffer = ::uniffi::MetadataBuffer::from_code(::uniffi::metadata::codes::TYPE_ENUM)
                .concat_str(#mod_path)
                .concat_str(#name);
        }

        #derive_ffi_traits
    }
}

pub(crate) fn error_meta_static_var(item: &EnumItem) -> syn::Result<TokenStream> {
    let name = item.name();
    let module_path = mod_path()?;
    let flat = item.is_flat_error();
    let non_exhaustive = item.is_non_exhaustive();
    let docstring = item.docstring();
    let mut metadata_expr = quote! {
            ::uniffi::MetadataBuffer::from_code(::uniffi::metadata::codes::ENUM)
                .concat_str(#module_path)
                .concat_str(#name)
                .concat_option_bool(Some(#flat))
                .concat_bool(false) // discr_type: None
    };
    if flat {
        metadata_expr.extend(flat_error_variant_metadata(item)?)
    } else {
        metadata_expr.extend(variant_metadata(item)?);
    }
    metadata_expr.extend(quote! {
        .concat_bool(#non_exhaustive)
        .concat_long_str(#docstring)
    });
    Ok(create_metadata_items("error", &name, metadata_expr, None))
}

pub fn flat_error_variant_metadata(item: &EnumItem) -> syn::Result<Vec<TokenStream>> {
    let enum_ = item.enum_();
    let variants_len =
        try_metadata_value_from_usize(enum_.variants.len(), "UniFFI limits enums to 256 variants")?;
    std::iter::once(Ok(quote! { .concat_value(#variants_len) }))
        .chain(enum_.variants.iter().map(|v| {
            let name = ident_to_string(&v.ident);
            let docstring = extract_docstring(&v.attrs)?;
            Ok(quote! {
                .concat_str(#name)
                .concat_long_str(#docstring)
            })
        }))
        .collect()
}
