/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use uniffi_meta::{checksum, FnMetadata, Metadata};

mod metadata;
mod scaffolding;

use self::{metadata::gen_metadata, scaffolding::gen_fn_scaffolding};

// TODO(jplatte): Ensure no generics, no async, …
// TODO(jplatte): Aggregate errors instead of short-circuiting, whereever possible

enum ExportItem {
    Function {
        item: syn::ItemFn,
        metadata: FnMetadata,
    },
}

pub fn expand_export(item: syn::Item, mod_path: &[String]) -> syn::Result<TokenStream> {
    match gen_metadata(item, mod_path)? {
        ExportItem::Function { item, metadata } => {
            let checksum = checksum(&metadata);
            let meta_static_var = create_metadata_static_var(&item.sig.ident, metadata.into());
            let scaffolding = gen_fn_scaffolding(&item, mod_path, checksum)?;

            Ok(quote! {
                #scaffolding
                #meta_static_var
            })
        }
    }
}

fn create_metadata_static_var(name: &Ident, val: Metadata) -> TokenStream {
    let data: Vec<u8> = bincode::serialize(&val).expect("Error serializing metadata item");
    let count = data.len();
    let var_name = format_ident!("UNIFFI_META_{}", name);

    quote! {
        #[no_mangle]
        pub static #var_name: [u8; #count] = [#(#data),*];
    }
}
