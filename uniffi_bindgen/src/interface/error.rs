/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! # Error definitions for a `ComponentInterface`.
//!
//! This module converts error definition from UDL into structures that can be
//! added to a `ComponentInterface`. A declaration in the UDL like this:
//!
//! ```
//! # let ci = uniffi_bindgen::interface::ComponentInterface::from_webidl(r##"
//! # namespace example {};
//! [Error]
//! enum Example {
//!   "one",
//!   "two"
//! };
//! # "##)?;
//! # Ok::<(), anyhow::Error>(())
//! ```
//!
//! Will result in an [`Error`] member being added to the resulting [`ComponentInterface`]:
//!
//! ```
//! # let ci = uniffi_bindgen::interface::ComponentInterface::from_webidl(r##"
//! # namespace example {};
//!  # [Error]
//! # enum Example {
//! #   "one",
//! #   "two"
//! # };
//! # "##)?;
//! let err = ci.get_error_definition("Example").unwrap();
//! assert_eq!(err.name(), "Example");
//! assert_eq!(err.values().len(), 2);
//! assert_eq!(err.values()[0], "one");
//! assert_eq!(err.values()[1], "two");
//! # Ok::<(), anyhow::Error>(())
//! ```
use std::convert::TryFrom;

use anyhow::{bail, Result};

use super::{APIConverter, ComponentInterface};

/// Represents an Error that might be thrown by functions/methods in the component interface.
///
/// Errors are represented in the UDL as enums with the special `[Error]` attribute, but
/// they're handled in the FFI very differently. We represent them using the `ffi_support::ExternError`
/// struct and assign an integer error code to each variant.
#[derive(Debug, Clone, Default, Hash)]
pub struct Error {
    pub(super) name: String,
    pub(super) values: Vec<String>,
    pub(super) docs: Vec<String>,
}

impl Error {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn values(&self) -> Vec<&str> {
        self.values.iter().map(|v| v.as_str()).collect()
    }

    pub fn docs(&self) -> Vec<&str> {
        self.docs.iter().map(|v| v.as_str()).collect()
    }
}

impl APIConverter<Error> for weedle::EnumDefinition<'_> {
    fn convert(&self, _ci: &mut ComponentInterface) -> Result<Error> {
        Ok(Error {
            name: self.identifier.0.to_string(),
            values: self
                .values
                .body
                .list
                .iter()
                .map(|v| v.0.to_string())
                .collect(),
            docs: vec![],
        })
    }
}

impl APIConverter<Error> for &syn::ItemEnum {
    fn convert(&self, _ci: &mut ComponentInterface) -> Result<Error> {
        let attrs = super::synner::Attributes::try_from(&self.attrs)?;
        let mut docs = attrs.docs;
        Ok(Error {
            name: self.ident.to_string(),
            values: self
                .variants
                .iter()
                .map(|v| {
                    let attrs = super::synner::Attributes::try_from(&v.attrs)?;
                    if v.discriminant.is_some() {
                        bail!("Explicit enum discriminants are not supported");
                    }
                    if !matches!(v.fields, syn::Fields::Unit) {
                        bail!("Error enum variants cannot currently have fields");
                    }
                    if attrs.docs.len() > 0 {
                        docs.push(String::from(""));
                        docs.push(format!("  `{}`:", v.ident.to_string()));
                        docs.extend(attrs.docs.iter().map(|ln| format!("      {}", ln)));
                    }
                    Ok(v.ident.to_string())
                })
                .collect::<Result<Vec<_>>>()?,
            docs,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_duplicate_variants() {
        const UDL: &str = r#"
            namespace test{};
            // Weird, but currently allowed!
            // We should probably disallow this...
            [Error]
            enum Testing { "one", "two", "one" };
        "#;
        let ci = ComponentInterface::from_webidl(UDL).unwrap();
        assert_eq!(ci.iter_error_definitions().len(), 1);
        assert_eq!(
            ci.get_error_definition("Testing").unwrap().values().len(),
            3
        );
    }
}
