/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use syn::{punctuated::Punctuated, spanned::Spanned, Attribute, Expr, Lit, Meta, Token};
use target_lexicon::Endianness;

use crate::Triple;

/// Compilation environment
///
/// This is use to skip or include items based on conditional compilation
pub struct CompileEnv {
    target: Triple,
    features: Vec<String>,
}

impl CompileEnv {
    pub fn new(target: Triple, features: Vec<String>) -> Self {
        Self { target, features }
    }

    #[cfg(test)]
    pub fn new_for_test() -> Self {
        Self {
            target: Triple::unknown(),
            features: vec![],
        }
    }

    pub fn should_parse_module(&self, attrs: &[Attribute]) -> syn::Result<bool> {
        // Use parse_attrs to just evaluate the `cfg` parts and ignore all the attrs.
        self.parse_attrs(attrs).map(|m| m.is_some())
    }

    /// Parse attributes, taking into account the `#[cfg]` and `#[cfg_attr]` logic.
    ///
    /// Returns None if a `#[cfg]` attribute fails to match.
    /// Otherwise, returns the list of `Meta` items left after applying the `#[cfg_attr]` logic.
    pub fn parse_attrs(&self, attrs: &[Attribute]) -> syn::Result<Option<Vec<Meta>>> {
        let mut metas = vec![];
        for a in attrs {
            if a.path().is_ident("cfg") {
                let child_meta: Meta = a.parse_args()?;
                if !self.check(&child_meta)? {
                    return Ok(None);
                }
            } else if a.path().is_ident("cfg_attr") {
                let nested = a.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;
                let mut items = nested.iter();
                let Some(first) = items.next() else {
                    return Err(syn::Error::new(a.span(), "predicate missing"));
                };
                if !self.check(first)? {
                    continue;
                }
                for child_attr in items {
                    metas.push(child_attr.clone());
                }
            } else {
                metas.push(a.meta.clone());
            }
        }
        Ok(Some(metas))
    }

    #[allow(clippy::match_like_matches_macro)]
    fn check(&self, meta: &Meta) -> syn::Result<bool> {
        let mut matched;
        let path = meta.path();
        if path.is_ident("all") {
            let child_metas = meta
                .require_list()?
                .parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;
            matched = true;
            for child_meta in child_metas {
                if !self.check(&child_meta)? {
                    matched = false;
                    break;
                }
            }
        } else if path.is_ident("any") {
            let child_metas = meta
                .require_list()?
                .parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;
            matched = false;
            for child_meta in child_metas {
                if self.check(&child_meta)? {
                    matched = true;
                    break;
                }
            }
        } else if path.is_ident("not") {
            let child_meta: Meta = meta.require_list()?.parse_args()?;
            matched = !self.check(&child_meta)?;
        } else if path.is_ident("feature") {
            let name = parse_meta_str(meta)?;
            matched = self.features.contains(&name);
        } else if path.is_ident("target_arch") {
            matched = self.target.architecture.to_string() == parse_meta_str(meta)?;
        } else if path.is_ident("target_os") {
            matched = self.target.operating_system.to_string() == parse_meta_str(meta)?;
        } else if path.is_ident("target_env") {
            matched = self.target.environment.to_string() == parse_meta_str(meta)?;
        } else if path.is_ident("target_vendor") {
            matched = self.target.vendor.to_string() == parse_meta_str(meta)?;
        } else if path.is_ident("target_endian") {
            let cfg_value = parse_meta_str(meta)?;
            matched = match (self.target.architecture.endianness(), cfg_value.as_str()) {
                (Ok(Endianness::Big), "big") => true,
                (Ok(Endianness::Little), "little") => true,
                _ => false,
            };
        } else if path.is_ident("target_pointer_width") {
            matched = match self.target.pointer_width() {
                Ok(p) => p.bits().to_string() == parse_meta_str(meta)?,
                Err(_) => false,
            };
        } else {
            path.is_ident("test");
            matched = false;
        }
        Ok(matched)
    }
}

fn parse_meta_str(meta: &Meta) -> syn::Result<String> {
    match &meta.require_name_value()?.value {
        Expr::Lit(e) => match &e.lit {
            Lit::Str(s) => Ok(s.value()),
            l => Err(syn::Error::new(l.span(), "expected string literal")),
        },
        v => Err(syn::Error::new(v.span(), "expected string literal")),
    }
}
