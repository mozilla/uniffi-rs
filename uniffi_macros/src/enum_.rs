use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Data, DeriveInput, Index};
use uniffi_meta::{EnumMetadata, FieldMetadata, VariantMetadata};

use crate::{
    export::metadata::convert::convert_type,
    util::{assert_type_eq, create_metadata_static_var, try_read_field},
};

pub fn expand_enum(input: DeriveInput, module_path: Vec<String>) -> TokenStream {
    let variants = match input.data {
        Data::Enum(e) => Some(e.variants),
        _ => None,
    };

    let ident = &input.ident;

    let (write_impl, try_read_impl) = match &variants {
        Some(variants) => {
            let write_match_arms = variants.iter().enumerate().map(|(i, v)| {
                let v_ident = &v.ident;
                let fields = v.fields.iter().map(|f| &f.ident);
                let idx = Index::from(i);
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

            let try_read_match_arms = variants.iter().enumerate().map(|(i, v)| {
                let idx = Index::from(i);
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

            (write_impl, try_read_impl)
        }
        None => {
            let unimplemented = quote! { ::std::unimplemented!() };
            (unimplemented.clone(), unimplemented)
        }
    };

    let meta_static_var = variants
        .map(|variants| {
            let name = ident.to_string();
            let variants_res: syn::Result<_> = variants
                .iter()
                .map(|v| {
                    let name = v.ident.to_string();
                    let fields = v
                        .fields
                        .iter()
                        .map(|f| {
                            let name = f
                                .ident
                                .as_ref()
                                .ok_or_else(|| {
                                    syn::Error::new_spanned(
                                        v,
                                        "UniFFI only supports enum variants with named fields \
                                         (or no fields at all)",
                                    )
                                })?
                                .to_string();

                            Ok(FieldMetadata {
                                name,
                                ty: convert_type(&f.ty)?,
                            })
                        })
                        .collect::<syn::Result<_>>()?;

                    Ok(VariantMetadata { name, fields })
                })
                .collect();

            match variants_res {
                Ok(variants) => {
                    let metadata = EnumMetadata {
                        module_path,
                        name,
                        variants,
                    };

                    create_metadata_static_var(ident, metadata.into())
                }
                Err(e) => e.into_compile_error(),
            }
        })
        .unwrap_or_else(|| {
            syn::Error::new(Span::call_site(), "This derive must only be used on enums")
                .into_compile_error()
        });

    let type_assertion = assert_type_eq(ident, quote! { crate::uniffi_types::#ident });

    quote! {
        impl ::uniffi::RustBufferFfiConverter for #ident {
            type RustType = Self;

            fn write(obj: Self, buf: &mut ::std::vec::Vec<u8>) {
                #write_impl
            }

            fn try_read(buf: &mut &[::std::primitive::u8]) -> ::uniffi::deps::anyhow::Result<Self> {
                #try_read_impl
            }
        }

        #meta_static_var
        #type_assertion
    }
}

pub fn write_field(f: &syn::Field) -> TokenStream {
    let ident = &f.ident;
    let ty = &f.ty;

    quote! {
        <#ty as ::uniffi::FfiConverter>::write(#ident, buf);
    }
}
