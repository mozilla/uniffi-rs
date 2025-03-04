/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

mod ast;
mod kw;
mod merge;
mod node;
mod parse;
mod render;

use ast::{ConstructNodeInput, DefineIrPassInput, IrPassInput, Items};
use proc_macro2::TokenStream;
use quote::quote;
use render::Ir;

pub use node::expand_node;

pub fn expand_ir(input: Items) -> syn::Result<TokenStream> {
    let items = input
        .nodes
        .values()
        .map(|node| node.render(Ir::NonPass))
        .collect::<TokenStream>();
    let pass_items = input
        .nodes
        .values()
        .map(|node| node.render(Ir::PassInput))
        .collect::<TokenStream>();
    let impls = input
        .impls
        .iter()
        .map(|impl_block| quote! { #impl_block })
        .collect::<TokenStream>();

    Ok(quote! {
        #items
        #impls

        #[allow(unused)]
        macro_rules! ir_add {
            ($macro_name:path, $($tt:tt)*) => {
                $macro_name! {
                    $($tt)*
                    {
                        #pass_items
                        #impls
                    }
                }
            };
        }
        // Add a use statement for the macro itself, so it can be accessed via the module path.
        pub(crate) use ir_add;
    })
}

pub fn expand_ir_pass(input: IrPassInput) -> syn::Result<TokenStream> {
    let IrPassInput { from, to } = input;

    Ok(quote! {
        #from::ir_add! {
            #to::ir_add,
            ::uniffi_internal_macros::define_ir_pass,
            { #from #to }
        }

    })
}

pub fn expand_define_ir_pass(input: DefineIrPassInput) -> syn::Result<TokenStream> {
    let input = input.render()?;
    Ok(quote! { #input })
}

pub fn expand_construct_node(input: ConstructNodeInput) -> syn::Result<TokenStream> {
    Ok(input.render())
}

fn ir_mod() -> proc_macro2::TokenStream {
    match std::env::var("CARGO_PKG_NAME") {
        Ok(s) if s == "uniffi_bindgen" => {
            quote::quote! { crate::ir }
        }
        _ => quote::quote! { ::uniffi_bindgen::ir },
    }
}
