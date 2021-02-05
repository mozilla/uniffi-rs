/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! # Enum definitions for a `ComponentInterface`.
//!
//! This module converts enum definition from UDL into structures that can be
//! added to a `ComponentInterface`. A declaration in the UDL like this:
//!
//! ```
//! # let ci = uniffi_bindgen::interface::ComponentInterface::from_webidl(r##"
//! # namespace example {};
//! enum Example {
//!   "one",
//!   "two"
//! };
//! # "##)?;
//! # Ok::<(), anyhow::Error>(())
//! ```
//!
//! Will result in a [`Enum`] member being added to the resulting [`ComponentInterface`]:
//!
//! ```
//! # let ci = uniffi_bindgen::interface::ComponentInterface::from_webidl(r##"
//! # namespace example {};
//! # enum Example {
//! #   "one",
//! #   "two"
//! # };
//! # "##)?;
//! let e = ci.get_enum_definition("Example").unwrap();
//! assert_eq!(e.name(), "Example");
//! assert_eq!(e.variants().len(), 2);
//! assert_eq!(e.variants()[0], "one");
//! assert_eq!(e.variants()[1], "two");
//! # Ok::<(), anyhow::Error>(())
//! ```

use anyhow::Result;

use super::{APIConverter, ComponentInterface};

/// Represents a simple C-style enum, with named variants.
///
/// In the FFI these are turned into a plain u32, with variants numbered
/// in the order they appear in the declaration, starting from 1.
#[derive(Debug, Clone, Hash)]
pub struct Enum {
    pub(super) name: String,
    pub(super) variants: Vec<String>,
}

impl Enum {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn variants(&self) -> Vec<&str> {
        self.variants.iter().map(|v| v.as_str()).collect()
    }
}

impl APIConverter<Enum> for weedle::EnumDefinition<'_> {
    fn convert(&self, _ci: &mut ComponentInterface) -> Result<Enum> {
        Ok(Enum {
            name: self.identifier.0.to_string(),
            variants: self
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
    fn test_duplicate_variants() -> Result<()> {
        const UDL: &str = r#"
            namespace test{};
            // Weird, but currently allowed!
            // We should probably disallow this...
            enum Testing { "one", "two", "one" };
        "#;
        let ci = ComponentInterface::from_webidl(UDL).unwrap();
        assert_eq!(ci.iter_enum_definitions().len(), 1);
        assert_eq!(
            ci.get_enum_definition("Testing").unwrap().variants().len(),
            3
        );
        Ok(())
    }
}
