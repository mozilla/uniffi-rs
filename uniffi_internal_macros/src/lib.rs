/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse_macro_input, Attribute, Data, DeriveInput, Expr, ExprLit, Fields, Index, Lit, Meta,
};

fn has_ignore_attribute(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| {
        if attr.path().is_ident("checksum_ignore") {
            if let Meta::List(_) | Meta::NameValue(_) = &attr.meta {
                panic!("#[checksum_ignore] doesn't accept extra information");
            }
            true
        } else {
            false
        }
    })
}

/// Custom derive for uniffi_meta::Checksum
#[proc_macro_derive(Checksum, attributes(checksum_ignore))]
pub fn checksum_derive(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);

    let name = input.ident;

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let code = match input.data {
        Data::Enum(enum_)
            if enum_.variants.len() == 1
                && enum_
                    .variants
                    .iter()
                    .all(|variant| matches!(variant.fields, Fields::Unit)) =>
        {
            quote!()
        }
        Data::Enum(enum_) => {
            let mut next_discriminant = 0u64;
            let match_inner = enum_.variants.iter().map(|variant| {
                let ident = &variant.ident;
                if has_ignore_attribute(&variant.attrs) {
                    panic!("#[checksum_ignore] is not supported in enums");
                }
                match &variant.discriminant {
                    Some((_, Expr::Lit(ExprLit { lit: Lit::Int(value), .. }))) => {
                        next_discriminant = value.base10_parse::<u64>().unwrap();
                    }
                    Some(_) => {
                        panic!("#[derive(Checksum)] doesn't support non-numeric explicit discriminants in enums");
                    }
                    None => {}
                }
                let discriminant = quote! { state.write(&#next_discriminant.to_le_bytes()) };
                next_discriminant += 1;
                match &variant.fields {
                    Fields::Unnamed(fields) => {
                        let field_idents = fields
                            .unnamed
                            .iter()
                            .enumerate()
                            .map(|(num, _)| format_ident!("__self_{}", num));
                        let field_stmts = field_idents
                            .clone()
                            .map(|ident| quote! { Checksum::checksum(#ident, state); });
                        quote! {
                            Self::#ident(#(#field_idents,)*) => {
                                #discriminant;
                                #(#field_stmts)*
                            }
                        }
                    }
                    Fields::Named(fields) => {
                        let field_idents = fields
                            .named
                            .iter()
                            .map(|field| field.ident.as_ref().unwrap());
                        let field_stmts = fields.named.iter()
                            .filter(|field| !has_ignore_attribute(&field.attrs))
                            .map(|field| {
                                    let ident = field.ident.as_ref().unwrap();
                                    quote! { Checksum::checksum(#ident, state); }
                            });
                        quote! {
                            Self::#ident { #(#field_idents,)* } => {
                                #discriminant;
                                #(#field_stmts)*
                            }
                        }
                    }
                    Fields::Unit => quote! { Self::#ident => #discriminant, },
                }
            });
            quote! {
                match self {
                    #(#match_inner)*
                }
            }
        }
        Data::Struct(struct_) => {
            let stmts = struct_
                .fields
                .iter()
                .enumerate()
                .filter(|&(_num, field)| (!has_ignore_attribute(&field.attrs)))
                .map(|(num, field)| match field.ident.as_ref() {
                    Some(ident) => quote! { Checksum::checksum(&self.#ident, state); },
                    None => {
                        let i = Index::from(num);
                        quote! { Checksum::checksum(&self.#i, state); }
                    }
                });
            quote! {
                #(#stmts)*
            }
        }
        Data::Union(_) => {
            panic!("#[derive(Checksum)] is not supported for unions");
        }
    };

    quote! {
        #[automatically_derived]
        impl #impl_generics Checksum for #name #ty_generics #where_clause {
            fn checksum<__H: ::core::hash::Hasher>(&self, state: &mut __H) {
                #code
            }
        }
    }
    .into()
}

/// Custom derive for AsType traits
///
/// Bindings generators will typically define their own `Type` struct and an `AsType` trait that
/// maps Bindings IR nodes to that type struct.  The details of these vary by language, but this
/// macro can derive the `AsType` trait for typical type layouts.
///
///
/// * For structs with a `ty` or `self_type` field, `as_type()` map to that field.
/// * For new-type style enums where each variant stores another struct, `as_type()` will map to
///   those struct's `as_type()` method.
///
/// See the Python bindings for an example of this in the wild.
#[proc_macro_derive(AsType)]
pub fn as_type_derive(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    let name = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let code = match input.data {
        Data::Struct(struct_) => {
            let potential_fields = struct_
                .fields
                .iter()
                .filter(|field| match field.ident.as_ref() {
                    None => false,
                    Some(i) => {
                        let name = i.to_string();
                        name == "ty" || name == "self_type"
                    }
                })
                .collect::<Vec<_>>();
            match potential_fields.len() {
                0 => panic!("#[derive(AsType)] requires a `ty` or `self_type` field"),
                1 => {
                    let ident = &potential_fields[0].ident;
                    quote! {
                        &self.#ident
                    }
                }
                _ => panic!("#[derive(AsType)] both `ty` and `self_type` defined"),
            }
        }
        Data::Enum(enum_) => {
            let match_inner = enum_.variants.iter().map(|variant| {
                let ident = &variant.ident;
                match &variant.fields {
                    Fields::Unnamed(fields) => {
                        if fields.unnamed.len() != 1 {
                            panic!("#[derive(AsType)] enum variants must have exactly 1 field")
                        }
                        quote! {
                            Self::#ident(inner) => inner.as_type(),
                        }
                    }
                    Fields::Named(fields) => {
                        if fields.named.len() != 1 {
                            panic!("#[derive(AsType)] enum variants must have exactly 1 field")
                        }
                        let field_ident = fields.named[0].ident.as_ref().unwrap();
                        quote! {
                            Self::#ident { #field_ident } => #field_ident.as_type(),
                        }
                    }
                    Fields::Unit => {
                        panic!("#[derive(AsType)] enum variants must have exactly 1 field");
                    }
                }
            });
            quote! {
                match self {
                    #(#match_inner)*
                }
            }
        }
        Data::Union(_) => {
            panic!("#[derive(AsType)] is not supported for unions");
        }
    };

    quote! {
        #[automatically_derived]
        impl #impl_generics AsType for #name #ty_generics #where_clause {
            fn as_type(&self) -> &Type {
                #code
            }
        }
    }
    .into()
}

/// Custom derive for AsCallable traits
///
/// AsCallable works similarly to AsType, except it maps nodes to the `Callable` struct.
/// Again, each bindings generator will define their own `Callable` struct and `AsCallable` trait.
/// This macro can auto-implement that trait for typical type layouts.
///
/// This macro only works for structs and is implemented by mapping `&self` -> `&self.callable`
#[proc_macro_derive(AsCallable)]
pub fn as_callable_derive(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    let name = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    quote! {
        #[automatically_derived]
        impl #impl_generics AsCallable for #name #ty_generics #where_clause {
            fn as_callable(&self) -> &Callable {
                &self.callable
            }
        }
    }
    .into()
}
