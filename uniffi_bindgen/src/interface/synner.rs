/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Some helpers for working with Rust code parsed via `syn`.
//!
//!

use anyhow::{bail, Result};

pub(super) fn name_from_path(p: &syn::Path) -> Result<String> {
    if p.segments.len() != 1 {
        bail!("no multi-component paths thanks ({:?})", p);
    }
    name_from_path_segment(p.segments.first().unwrap())
}

pub(super) fn name_from_path_segment(p: &syn::PathSegment) -> Result<String> {
    if !p.arguments.is_empty() {
        bail!("Can't get name from path segument with arguments: {:?}", p)
    }
    Ok(p.ident.to_string())
}

pub(super) fn name_and_args_from_path(p: &syn::Path) -> Result<(String, Vec<&syn::Type>)> {
    if p.segments.len() != 1 {
        bail!("no multi-component paths thanks ({:?})", p);
    }
    name_and_args_from_path_segment(p.segments.first().unwrap())
}

pub(super) fn name_and_args_from_path_segment(
    p: &syn::PathSegment,
) -> Result<(String, Vec<&syn::Type>)> {
    Ok((
        p.ident.to_string(),
        match &p.arguments {
            syn::PathArguments::None => vec![],
            syn::PathArguments::AngleBracketed(args) => args
                .args
                .iter()
                .map(|arg| match arg {
                    syn::GenericArgument::Type(t) => Ok(t),
                    _ => bail!("Path argument not supported {:?}", arg),
                })
                .collect::<Result<Vec<_>>>()?,
            syn::PathArguments::Parenthesized(_) => {
                bail!("Parenthesized path arguments not supported")
            }
        },
    ))
}

pub(super) fn name_from_type(t: &syn::Type) -> Result<String> {
    match t {
        syn::Type::Path(p) => {
            if p.qself.is_some() {
                bail!("no qualified-self paths thanks")
            }
            name_from_path(&p.path)
        }
        _ => bail!("Could not get name from type expression {:?}", t),
    }
}

pub(super) fn name_from_pattern(p: &syn::Pat) -> Result<String> {
    Ok(match p {
        syn::Pat::Ident(v) => {
            // TODO: check against presence of other stuff in the patter
            v.ident.to_string()
        }
        _ => bail!("Unable to get identifier from pattern {:?}", p),
    })
}

pub(super) fn name_pair_from_path(p: &syn::Path) -> Result<(String, String)> {
    if p.segments.len() != 2 {
        bail!("expected exactly two path components")
    }
    Ok((
        p.segments[0].ident.to_string(),
        p.segments[1].ident.to_string(),
    ))
}

pub(super) fn name_from_meta(m: &syn::Meta) -> Result<String> {
    match m {
        syn::Meta::Path(p) => name_from_path(p),
        _ => bail!("Can't get name from non-path Meta item"),
    }
}

pub(super) fn names_from_meta_list(m: &syn::MetaList) -> Result<Vec<String>> {
    m.nested
        .iter()
        .map(|m| match m {
            syn::NestedMeta::Meta(n) => name_from_meta(n),
            _ => bail!("Can't get name from nested meta-list item"),
        })
        .collect::<Result<Vec<_>>>()
}

pub(super) fn destructure_if_result_type(t: &syn::Type) -> Result<(Option<String>, &syn::Type)> {
    Ok(match t {
        syn::Type::Path(p) => {
            match name_and_args_from_path(&p.path) {
                Err(_) => (None, t),
                Ok((name, args)) => {
                    // TODO: hrm, how to support Result<T> shortcut syntax..?
                    if name != "Result" || args.len() != 2 {
                        (None, t)
                    } else {
                        (Some(name_from_type(args[1])?), args[0])
                    }
                }
            }
        }
        _ => bail!("Could not destructure possible result type  {:?}", t),
    })
}

/// Container for the limited set of item attributes supported in a UniFFI component crate.
///
/// This struct helps us to parse useful attributes from an item that help define how it
/// behaves in a component interface, and also to error out if we encounter any attributes
/// that we don't know how to interpret.
#[derive(Default)]
pub(super) struct Attributes {
    pub docs: Vec<String>,
    pub is_error: bool,
    // pub is_interface_declaration: bool,
}

impl std::convert::TryFrom<&Vec<syn::Attribute>> for Attributes {
    type Error = anyhow::Error;
    fn try_from(attributes: &Vec<syn::Attribute>) -> Result<Self> {
        let mut attrs = <Attributes as Default>::default();
        for attr in attributes {
            match attr.path.segments.len() {
                1 => {
                    let name = name_from_path(&attr.path)?;
                    match name.as_str() {
                        // Extract doc comments, they're very useful!
                        "doc" => match attr.parse_meta()? {
                            syn::Meta::NameValue(v) => match v.lit {
                                syn::Lit::Str(s) => {
                                    attrs.docs.push(s.value());
                                }
                                _ => bail!("Unexpected format for doc attribute"),
                            },
                            _ => bail!("Unexpected format for doc attribute"),
                        },
                        // Allow a handful of well-understood derives.
                        // TODO: we should guard against these being weird aliases.
                        "derive" => match attr.parse_meta()? {
                            syn::Meta::List(v) => {
                                for derived in names_from_meta_list(&v)? {
                                    match derived.as_str() {
                                        "Error" => attrs.is_error = true,
                                        "Debug" | "Clone" | "Default" | "Eq" | "PartialEq"
                                        | "Hash" | "Serialize" | "Deserialize" => (),
                                        _ => bail!("Unsupported derive: {}", derived),
                                    }
                                }
                            }
                            _ => bail!("Unexpected format for derive attribute"),
                        },
                        // Ignore clippy lints.
                        "allow" | "warn" | "deny" => (),
                        // Ignore error variant annotations from `thiserror`.
                        "error" => (),
                        // Ignore serde for now, although I don't like this on the public API.
                        "serde" => (),
                        _ => bail!("Unexpected item attribute {:#?}", attr),
                    }
                }
                2 => {
                    let (mod_name, macro_name) = name_pair_from_path(&attr.path)?;
                    match (mod_name.as_str(), macro_name.as_str()) {
                        ("uniffi", "declare_interface") => {
                            // attrs.is_interface_declaration = true;
                        }
                        _ => bail!("Unexpected item attribute {:#?}", attr),
                    }
                }
                _ => bail!("Unexpected item attribute {:#?}", attr),
            }
        }
        Ok(attrs)
    }
}
