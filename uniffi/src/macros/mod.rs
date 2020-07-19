/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use quote::{format_ident, quote};
use serde::{Deserialize, Serialize};
use std::convert::Into;

pub mod export_class;
pub mod export_enum;
pub mod export_fn;
pub mod export_methods;
pub mod export_record;

pub use export_class::ClassDefinition;
pub use export_enum::EnumDefinition;
pub use export_fn::FunctionDefinition;
pub use export_methods::{MethodDefinition, MethodDefinitions};
pub use export_record::RecordDefinition;

#[derive(Serialize, Deserialize)]
pub enum ExportDefinition {
    Enum(EnumDefinition),
    Function(FunctionDefinition),
    Record(RecordDefinition),
    Class(ClassDefinition),
    Method(MethodDefinition),
}

impl ExportDefinition {
    pub fn to_bincode(&self) -> Vec<u8> {
        // Serialization of this struct is infallible.
        bincode::serialize(self).unwrap()
    }

    pub fn export_name(&self) -> &str {
        match self {
            Self::Enum(defn) => defn.name(),
            Self::Function(defn) => defn.name(),
            _ => panic!("not implemented yet"),
        }
    }
}

impl quote::ToTokens for ExportDefinition {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let tt: proc_macro2::TokenStream = self.into();
        tt.to_tokens(tokens);
    }
}

impl Into<proc_macro2::TokenStream> for &ExportDefinition {
    fn into(self) -> proc_macro2::TokenStream {
        let name = format_ident!("UNIFFI_EXPORT_{}", self.export_name());
        let data = self.to_bincode();
        let data_len = syn::Index::from(data.len());
        let data_bytes = data.iter().map(|i| syn::Index::from(*i as usize));
        quote! {
            #[no_mangle]
            #[cfg_attr(any(target_os="macos", target_os="ios"), link_section = "DATA,.uniffi_idl")]
            #[cfg_attr(not(any(target_os="macos", target_os="ios")), link_section = ".uniffi_idl")]
            pub static #name: [u8;#data_len] = [#(#data_bytes),*];
        }
    }
}

#[macro_export]
macro_rules! syn_err {
    ($item:expr, $msg:expr) => {
        Err(syn::Error::new($item.span(), $msg))
    };
}
