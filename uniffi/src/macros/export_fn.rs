/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use quote::{format_ident, quote};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::convert::{Into, TryFrom};
use syn::spanned::Spanned;

use crate::syn_err;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionDefinition {
    name: String,
    arguments: Vec<ArgumentDefinition>,
    return_type: Option<TypeReference>,
}

impl FunctionDefinition {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn arguments(&self) -> Vec<&ArgumentDefinition> {
        self.arguments.iter().collect()
    }
    pub fn return_type(&self) -> Option<&TypeReference> {
        self.return_type.as_ref()
    }
}

impl TryFrom<&syn::ItemFn> for FunctionDefinition {
    type Error = syn::Error;
    fn try_from(item: &syn::ItemFn) -> syn::Result<Self> {
        if !matches!(item.vis, syn::Visibility::Public(_)) {
            return syn_err!(item, "Exported functions must be public");
        }
        if item.sig.asyncness.is_some() {
            return syn_err!(item, "Functions marked `async` cannot be exported");
        }
        if item.sig.unsafety.is_some() {
            return syn_err!(item, "Functions marked `unsafe` cannot be exported");
        }
        if item.sig.abi.is_some() {
            return syn_err!(item, "Functions marked `extern` cannot be exported");
        }
        if !item.sig.generics.params.is_empty() || item.sig.generics.where_clause.is_some() {
            return syn_err!(
                item.sig.generics,
                "Exported functions cannot have generic parameters"
            );
        }
        if item.sig.variadic.is_some() {
            return syn_err!(item, "Functions with variadic arguments cannot be exported");
        }
        Ok(FunctionDefinition {
            name: item.sig.ident.to_string(),
            arguments: item
                .sig
                .inputs
                .iter()
                .map(|arg| match &arg {
                    syn::FnArg::Receiver(_) => syn_err!(
                        arg,
                        "Exported function cannot have a method receiver argument"
                    ),
                    syn::FnArg::Typed(arg) => Ok(ArgumentDefinition::try_from(arg)?),
                })
                .collect::<syn::Result<Vec<_>>>()?,
            return_type: match &item.sig.output {
                syn::ReturnType::Default => None,
                syn::ReturnType::Type(_, type_) => Some(TypeReference::try_from(type_.as_ref())?),
            },
        })
    }
}

impl Into<proc_macro2::TokenStream> for &FunctionDefinition {
    fn into(self) -> proc_macro2::TokenStream {
        let name = format_ident!("{}", self.name);
        // XXX TODO: get namespacing info from calling crate.
        let extern_name = format_ident!("arithmetic_{}", self.name);
        // Each argument is received over the FFI as the type of its `ViaFfi` impl.
        let args_recv = self.arguments.iter().map(|arg| {
            let nm = format_ident!("{}", arg.name);
            let ty = &arg.type_;
            quote! {
              #nm: <#ty as uniffi::support::ViaFfi>::Value
            }
        });
        // To forward each argument on to the rust call, convert it with `try_from_ffi_value()`.
        let args_call = self.arguments.iter().map(|arg| {
            let nm = format_ident!("{}", arg.name);
            let ty = &arg.type_;
            quote! {
              <#ty as uniffi::support::ViaFfi>::try_from_ffi_value(#nm).unwrap()
            }
        });
        // The body of the `extern "C"` fn, which depends on whether there's a return type.
        // In the future we'll probably have more work to do here, e.g. if it returns an error.
        let extern_fn = match &self.return_type {
            None => quote! {
               pub extern "C" fn #extern_name(#(#args_recv),*) {
                   #name(#(#args_call),*);
               }
            },
            Some(ret) => quote! {
                pub extern "C" fn #extern_name(#(#args_recv),*) -> #ret {
                    <#ret as uniffi::support::ViaFfi>::into_ffi_value(
                      #name(#(#args_call),*)
                    )
                }
            },
        };
        // The exported `FunctionDefinition` struct will only be valid if the argument type names
        // we discovered from the rust code actually match the names those types use in the generated
        // API. Let's not compile code where that isn't true.
        let mut named_type_names: HashSet<&str> = self
            .arguments
            .iter()
            .filter_map(|arg| {
                if let TypeReference::Named(nm) = &arg.type_ {
                    Some(nm.as_str())
                } else {
                    None
                }
            })
            .collect();
        if let Some(ty) = &self.return_type {
            if let TypeReference::Named(nm) = ty {
                named_type_names.insert(nm.as_str());
            }
        }
        let named_type_names = named_type_names.iter().collect::<Vec<_>>();
        let named_type_idents = named_type_names.iter().map(|nm| format_ident!("{}", nm));
        // Finally, we need to include the FunctionDefinition itself in the generated lib.
        let serialized_defn = super::ExportDefinition::Function(self.clone());
        quote! {
            #[no_mangle]
            #extern_fn
            #(uniffi_macros::uniffi_assert_type_name!(#named_type_idents, #named_type_names);)*
            #serialized_defn
        }
    }
}

impl quote::ToTokens for FunctionDefinition {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let tt: proc_macro2::TokenStream = self.into();
        tt.to_tokens(tokens);
    }
}

// Represents an argument to a function/constructor/method call.
//
// Each argument has a name and a type, along with some optional metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArgumentDefinition {
    name: String,
    type_: TypeReference,
}

impl TryFrom<&syn::PatType> for ArgumentDefinition {
    type Error = syn::Error;
    fn try_from(item: &syn::PatType) -> syn::Result<Self> {
        let name = match &*item.pat {
            syn::Pat::Ident(pat) => {
                if pat.by_ref.is_some() {
                    return syn_err!(
                        item,
                        "Exported function arguments cannot be passed by reference (yet)"
                    );
                }
                if pat.subpat.is_some() {
                    return syn_err!(
                        item,
                        "Exported function arguments must have simple names, nothing fancy!"
                    );
                }
                pat.ident.to_string()
            }
            _ => {
                println!("UNSUPPORTED ARG PATTERN {:?}", item);
                return syn_err!(
                    item,
                    "Exported function arguments must have simple names, nothing fancy!"
                );
            }
        };
        Ok(ArgumentDefinition {
            name,
            type_: TypeReference::try_from(item.ty.as_ref())?,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TypeReference {
    Bool,
    U8,
    I8,
    U16,
    I16,
    U32,
    I32,
    U64,
    I64,
    F32,
    F64,
    String,
    Bytes,
    Named(String),
    //Optional(Box<TypeReference>),
    //Sequence(Box<TypeReference>),
}

impl TryFrom<&syn::Type> for TypeReference {
    type Error = syn::Error;
    fn try_from(item: &syn::Type) -> syn::Result<Self> {
        match item {
            syn::Type::Path(typ) => {
                if typ.qself.is_some() {
                    return syn_err!(
                        item,
                        "Exported types must have simple names, nothing fancy!"
                    );
                }
                if typ.path.leading_colon.is_some() {
                    return syn_err!(
                        item,
                        "Exported types must have simple names, nothing fancy!"
                    );
                }
                if typ.path.segments.len() != 1 {
                    return syn_err!(
                        item,
                        "Exported types must have simple names, nothing fancy!"
                    );
                }
                let path = typ.path.segments.first().unwrap();
                if !matches!(path.arguments, syn::PathArguments::None) {
                    return syn_err!(
                        item,
                        "Exported types must have simple names, nothing fancy!"
                    );
                }
                let nm = path.ident.to_string();
                Ok(match nm.as_str() {
                    "bool" => TypeReference::Bool,
                    "u8" => TypeReference::U8,
                    "i8" => TypeReference::I8,
                    "u16" => TypeReference::U16,
                    "i16" => TypeReference::I16,
                    "u32" => TypeReference::U32,
                    "i32" => TypeReference::I32,
                    "u64" => TypeReference::U64,
                    "i64" => TypeReference::I64,
                    "f32" => TypeReference::F32,
                    "f64" => TypeReference::F64,
                    _ => TypeReference::Named(nm),
                })
            }
            _ => {
                println!("UNSUPPORTED TYPE: {:?}", item);
                syn_err!(
                    item,
                    "Exported types must have simple names, nothing fancy!"
                )
            }
        }
    }
}

impl Into<proc_macro2::TokenStream> for &TypeReference {
    fn into(self) -> proc_macro2::TokenStream {
        let name = match self {
            TypeReference::Bool => "bool",
            TypeReference::U8 => "u8",
            TypeReference::I8 => "i8",
            TypeReference::U16 => "u16",
            TypeReference::I16 => "i16",
            TypeReference::U32 => "u32",
            TypeReference::I32 => "i32",
            TypeReference::U64 => "u64",
            TypeReference::I64 => "i64",
            TypeReference::F32 => "f32",
            TypeReference::F64 => "f64",
            TypeReference::String => "&str",
            TypeReference::Bytes => "&[u8]",
            TypeReference::Named(nm) => nm.as_str(),
        };
        let name = format_ident!("{}", name);
        quote! {
            #name
        }
    }
}

impl quote::ToTokens for TypeReference {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let tt: proc_macro2::TokenStream = self.into();
        tt.to_tokens(tokens);
    }
}
