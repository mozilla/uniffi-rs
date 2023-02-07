/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::{collections::HashMap, fs::read_to_string};

use anyhow::Result;
use camino::Utf8Path;
use syn::Attribute;

/// Function documentation.
#[derive(Debug, Clone)]
pub struct Function {
    pub description: String,
    pub arguments_descriptions: HashMap<String, String>,
    pub return_description: Option<String>,
}

/// Structure or enum documentation.
#[derive(Debug, Clone)]
pub struct Structure {
    pub description: String,
}

/// Impl documentation.
#[derive(Debug, Clone)]
pub struct Impl {
    pub methods: HashMap<String, Function>,
}

#[derive(Debug)]
pub struct Documentation {
    pub functions: HashMap<String, Function>,
    pub structures: HashMap<String, Structure>,
    pub impls: HashMap<String, Impl>,
}

/// Extract doc comment from attributes.
///
/// Rust doc comments are silently converted (during parsing) to attributes of form:
/// #[doc = "documentation comment content"]
fn extract_doc_comment(attrs: &[Attribute]) -> Option<String> {
    attrs
        .iter()
        .filter_map(|attr| {
            attr.parse_meta().ok().and_then(|meta| {
                if let syn::Meta::NameValue(named_value) = meta {
                    let is_doc = named_value.path.is_ident("doc");
                    if is_doc {
                        match named_value.lit {
                            syn::Lit::Str(comment) => Some(comment.value().trim().to_string()),
                            _ => None,
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
        })
        .next()
}

/// Extract code documentation comments from Rust `lib.rs` file.
pub fn extract_documentation(path: &Utf8Path) -> Result<Documentation> {
    let input = read_to_string(path)?;
    let file = syn::parse_file(&input)?;

    let mut functions = HashMap::new();
    let mut structures = HashMap::new();
    let mut impls = HashMap::new();

    for item in file.items.into_iter() {
        match item {
            syn::Item::Enum(item) => {
                let name = item.ident.to_string();
                let description = extract_doc_comment(&item.attrs);
                if let Some(description) = description {
                    structures.insert(name, Structure { description });
                }
            }
            syn::Item::Struct(item) => {
                let name = item.ident.to_string();
                let description = extract_doc_comment(&item.attrs);
                if let Some(description) = description {
                    structures.insert(name, Structure { description });
                }
            }
            syn::Item::Impl(item) => {
                if item.trait_.is_none() {
                    if let syn::Type::Path(path) = *item.self_ty {
                        let name = path.path.segments[0].ident.to_string();

                        let methods = item
                            .items
                            .into_iter()
                            .filter_map(|item| {
                                if let syn::ImplItem::Method(method) = item {
                                    let name = method.sig.ident.to_string();
                                    extract_doc_comment(&method.attrs).map(|doc| (name, doc))
                                } else {
                                    None
                                }
                            })
                            .map(|(name, description)| {
                                // todo: parse markdown to extract argument descriptions and return description
                                (
                                    name,
                                    Function {
                                        description,
                                        arguments_descriptions: HashMap::new(),
                                        return_description: None,
                                    },
                                )
                            })
                            .collect();

                        impls.insert(name, Impl { methods });
                    }
                }
            }
            syn::Item::Fn(item) => {
                let name = item.sig.ident.to_string();
                let description = extract_doc_comment(&item.attrs);
                if let Some(description) = description {
                    functions.insert(
                        name,
                        Function {
                            description,
                            arguments_descriptions: HashMap::new(),
                            return_description: None,
                        },
                    );
                }
            }
            _ => (), // other item types are ignored for now,
        }
    }

    Ok(Documentation {
        functions,
        structures,
        impls,
    })
}
