/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::{ast::*, ir_mod};
use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

pub fn expand_node(node: Node) -> syn::Result<TokenStream> {
    let node_impl = node_impl(&node);
    let from_uniffi_meta_impl = node
        .attrs
        .from_uniffi_meta
        .as_ref()
        .map(|uniffi_meta_type| from_uniffi_meta_impl(&node, uniffi_meta_type));
    Ok(quote! {
        #node_impl
        #from_uniffi_meta_impl
    })
}

fn from_uniffi_meta_impl(node: &Node, uniffi_meta_type: &Ident) -> TokenStream {
    let ir_mod = ir_mod();
    let type_name = &node.ident;
    let body = match &node.def {
        NodeDef::Struct(fields) => {
            let pattern = fields.uniffi_meta_type_pattern();
            let construct = fields.construct_from_uniffi_meta();
            quote! {
                let ::uniffi_meta::#uniffi_meta_type #pattern = node;
                Ok(#type_name #construct)
            }
        }
        NodeDef::Enum(variants) => {
            let cases = variants.variants.iter().map(|(variant_name, v)| {
                let pattern = v.fields.uniffi_meta_type_pattern();
                let construct = v.fields.construct_from_uniffi_meta();
                let uniffi_meta_variant = v.attrs.from_uniffi_meta.as_ref().unwrap_or(&v.ident);
                quote! {
                    ::uniffi_meta::#uniffi_meta_type::#uniffi_meta_variant #pattern => {
                        Ok(#type_name::#variant_name #construct)
                    }
                }
            });
            quote! {
                match node {
                    #(#cases)*
                }
            }
        }
    };
    quote! {
        #[automatically_derived]
        impl #ir_mod::FromNode<::uniffi_meta::#uniffi_meta_type> for #type_name {
            fn from_node(node: ::uniffi_meta::#uniffi_meta_type) -> ::anyhow::Result<#type_name> {
                use #ir_mod::IntoNode;
                #body
            }
        }
    }
}

fn node_impl(node: &Node) -> TokenStream {
    let ir_mod = ir_mod();
    let type_ident = &node.ident;
    let (impl_generics, ty_generics, where_clause) = node.generics.split_for_node_impl();
    let type_name = type_ident.to_string();
    let visit_children_body = visit_children_body(node);
    let empty_body = empty_body(node);

    quote! {
        #[automatically_derived]
        impl #impl_generics #ir_mod::Node for #type_ident #ty_generics #where_clause {
            fn visit_children(&self, visitor: &mut dyn FnMut(&str, &dyn Node) -> ::anyhow::Result<()>) -> ::anyhow::Result<()>
            {
                #visit_children_body
            }

            fn visit_children_mut(&mut self, visitor: &mut dyn FnMut(&str, &mut dyn Node) -> ::anyhow::Result<()>) -> ::anyhow::Result<()>
            {
                #visit_children_body
            }

            fn type_name(&self) -> Option<&'static str> {
                Some(#type_name)
            }

            fn as_any(&self) -> &dyn std::any::Any {
                self
            }

            fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
                self
            }

            fn empty() -> Self {
                use #ir_mod::Node;
                #empty_body
            }
        }
    }
}

fn visit_children_body(node: &Node) -> TokenStream {
    match &node.def {
        NodeDef::Struct(fields) => {
            let pattern = fields.pattern();
            let visits = visit_children_visits(None, fields);
            quote! {
                let Self #pattern = self;
                #(#visits)*
                Ok(())
            }
        }
        NodeDef::Enum(variants) => {
            let cases = variants.variants.values().map(|v| {
                let pattern = v.fields.pattern();
                let variant = &v.ident;
                let visits = visit_children_visits(Some(variant), &v.fields);
                quote! {
                    Self::#variant #pattern => {
                        #(#visits)*
                    }
                }
            });
            quote! {
                match self {
                    #(#cases)*
                }
                Ok(())
            }
        }
    }
}

/// Render the `visitor(...)` statements for `visit_children` and `visit_children_mut`
///
/// This works for both structs and variants.  Structs must have their variables unpacked using the
/// `pattern` function below
fn visit_children_visits(variant: Option<&Ident>, fields: &Fields) -> Vec<TokenStream> {
    let variant_name = match variant {
        Some(variant) => format!("::{variant}"),
        None => "".to_string(),
    };
    match fields {
        Fields::Unit => vec![],
        Fields::Named(fields) => fields
            .values()
            .map(|f| {
                let ident = &f.ident;
                let name = format!("{variant_name}.{ident}");
                quote! { visitor(#name, #ident)?; }
            })
            .collect(),
        Fields::Tuple(tuple_fields) => tuple_fields
            .iter()
            .enumerate()
            .map(|(i, f)| {
                let var = &f.var_name;
                let name = format!("{variant_name}.{i}");
                quote! { visitor(#name, #var)?; }
            })
            .collect(),
    }
}

fn empty_body(node: &Node) -> TokenStream {
    match &node.def {
        NodeDef::Struct(fields) => {
            let construct = fields.construct_empty();
            quote! { Self #construct }
        }
        NodeDef::Enum(variants) => {
            let Some(first_variant) = variants.variants.values().next() else {
                let type_name = node.ident.to_string();
                return quote! {
                    panic!("{} has no variants", #type_name)
                };
            };
            let first_variant_name = &first_variant.ident;
            let construct = first_variant.fields.construct_empty();
            quote! { Self::#first_variant_name #construct }
        }
    }
}
