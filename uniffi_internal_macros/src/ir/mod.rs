/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

mod ast;
mod parse;
mod render;

use proc_macro2::TokenStream;
use quote::quote;

use ast::*;

pub fn expand_node(node: Node) -> syn::Result<TokenStream> {
    let type_ident = &node.ident;
    let (impl_generics, ty_generics, where_clause) = node.split_generics_for_node_impl();
    let type_name = type_ident.to_string();
    let visit_children = node.render_visit_children();
    let visit_children_mut = node.render_visit_children_mut();
    let take_into_value = node.render_take_into_value();
    let try_from_value = node.render_try_from_value();
    let default = node.render_default();

    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics ::uniffi_pipeline::Node for #type_ident #ty_generics #where_clause {
            #visit_children

            #visit_children_mut

            #take_into_value

            #try_from_value

            fn type_name(&self) -> Option<&'static str> {
                Some(#type_name)
            }

            fn as_any(&self) -> &dyn ::std::any::Any {
                self
            }

            fn as_any_mut(&mut self) -> &mut dyn ::std::any::Any {
                self
            }

            fn to_box_any(self: Box<Self>) -> Box<dyn ::std::any::Any> {
                self
            }
        }

        #[automatically_derived]
        impl #impl_generics ::std::default::Default for #type_ident #ty_generics #where_clause {
            #default
        }
    })
}
