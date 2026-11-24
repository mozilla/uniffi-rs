/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use indexmap::IndexSet;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Data, DataEnum, DataStruct, DeriveInput, Ident, Result};

pub fn expand_derive(input: DeriveInput) -> Result<TokenStream> {
    match input.data {
        Data::Struct(st) => render_struct(input.ident, st),
        Data::Enum(en) => render_enum(input.ident, en),
        Data::Union(_) => panic!("#[derive(Node)] is not supported for unions"),
    }
}

fn render_struct(ident: Ident, st: DataStruct) -> Result<TokenStream> {
    let type_name = ident.to_string();
    let members = st.fields.members();
    // Deduplicate in declaration order: iterating a HashSet would emit the checks in an
    // order seeded per process, so each build would expand the derive differently.
    let unique_field_types = st
        .fields
        .iter()
        .map(|f| &f.ty)
        .collect::<IndexSet<_>>()
        .into_iter();

    Ok(quote! {
        #[automatically_derived]
        impl ::uniffi_pipeline::Node for #ident {
            fn type_name(&self) -> Option<&'static str> {
                Some(#type_name)
            }

            fn as_any(&self) -> &dyn ::std::any::Any {
                self
            }

            fn to_box_any(self: ::std::boxed::Box<Self>) -> ::std::boxed::Box<dyn ::std::any::Any> {
                self
            }

            fn visit_children<'a>(&'a self, visitor: &mut dyn FnMut(&'a dyn Node)) {
                #(
                    visitor(&self.#members);
                )*
            }

            fn has_descendant_type<N: Node>(visited: &mut ::std::collections::HashSet::<::std::any::TypeId>) -> bool {
                use std::any::TypeId;

                if TypeId::of::<N>() == TypeId::of::<Self>() {
                    return true
                }
                if !visited.insert(TypeId::of::<Self>()) {
                    return false;
                }
                #(
                    if <#unique_field_types as ::uniffi_pipeline::Node>::has_descendant_type::<N>(visited) {
                        return true
                    }
                )*
                false
            }
        }
    })
}

fn render_enum(ident: Ident, en: DataEnum) -> Result<TokenStream> {
    let type_name = ident.to_string();
    let arms = en.variants.iter().map(|v| {
        let vident = &v.ident;
        let members = v.fields.members();
        let vars = (0..v.fields.len())
            .map(|i| format_ident!("var{i}"))
            .collect::<Vec<_>>();
        quote! {
            Self::#vident { #(#members: #vars),* } => {
                #(
                    visitor(#vars);
                )*
            }
        }
    });

    let unique_field_types = en
        .variants
        .iter()
        .flat_map(|v| v.fields.iter().map(|f| &f.ty))
        .collect::<IndexSet<_>>()
        .into_iter();

    Ok(quote! {
        #[automatically_derived]
        impl ::uniffi_pipeline::Node for #ident {
            fn type_name(&self) -> Option<&'static str> {
                Some(#type_name)
            }

            fn as_any(&self) -> &dyn ::std::any::Any {
                self
            }

            fn to_box_any(self: ::std::boxed::Box<Self>) -> ::std::boxed::Box<dyn ::std::any::Any> {
                self
            }

            fn visit_children<'a>(&'a self, visitor: &mut dyn FnMut(&'a dyn Node)) {
                match self {
                    #(
                        #arms
                    )*
                }
            }

            fn has_descendant_type<N: Node>(visited: &mut ::std::collections::HashSet::<::std::any::TypeId>) -> bool {
                use std::any::TypeId;

                if TypeId::of::<N>() == TypeId::of::<Self>() {
                    return true
                }
                if !visited.insert(TypeId::of::<Self>()) {
                    return false;
                }
                #(
                    if <#unique_field_types as ::uniffi_pipeline::Node>::has_descendant_type::<N>(visited) {
                        return true
                    }
                )*
                false
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    // The field type named by each `has_descendant_type` check, in expansion order.
    fn check_order(input: DeriveInput) -> Vec<String> {
        let expanded = expand_derive(input).unwrap().to_string();
        expanded
            .match_indices(" as :: uniffi_pipeline :: Node >")
            .map(|(end, _)| {
                let start = expanded[..end].rfind('<').unwrap() + 1;
                expanded[start..end].trim().to_string()
            })
            .collect()
    }

    // With this many distinct types, a HashSet order matches declaration order only by
    // chance, so these fail on the order that made builds irreproducible.
    #[test]
    fn struct_checks_follow_declaration_order() {
        let order = check_order(parse_quote! {
            struct S { a: A, b: B, c: C, d: D, a2: A, e: E, f: F, g: G, h: H }
        });
        assert_eq!(order, ["A", "B", "C", "D", "E", "F", "G", "H"]);
    }

    #[test]
    fn enum_checks_follow_declaration_order() {
        let order = check_order(parse_quote! {
            enum E { One(A, B), Two { c: C, a: A }, Three(D, E, F), Four { g: G, h: H } }
        });
        assert_eq!(order, ["A", "B", "C", "D", "E", "F", "G", "H"]);
    }
}
