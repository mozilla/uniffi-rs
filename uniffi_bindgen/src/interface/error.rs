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
//! let values: Vec<_> = err.values().collect();
//! assert_eq!(values.len(), 2);
//! assert_eq!(values[0], "one");
//! assert_eq!(values[1], "two");
//! # Ok::<(), anyhow::Error>(())
//! ```

use anyhow::Result;

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
}

impl Error {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn values(&self) -> impl Iterator<Item = &str> {
        self.values.iter().map(|v| v.as_str())
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
        assert_eq!(ci.iter_error_definitions().count(), 1);
        assert_eq!(
            ci.get_error_definition("Testing").unwrap().values().count(),
            3
        );
    }
}
