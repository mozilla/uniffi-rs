/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! # Record definitions for a `ComponentInterface`.
//!
//! This module converts "dictionary" definitions from UDL into [`Record`] structures
//! that can be added to a `ComponentInterface`, which are the main way we define structured
//! data types for a UniFFI Rust Component. A [`Record`] has a fixed set of named fields,
//! each of a specific type.
//!
//! (The terminology mismatch between "dictionary" and "record" is a historical artifact
//! due to this tool being loosely inspired by WebAssembly Interface Types, which used
//! the term "record" for this sort of data).
//!
//! A declaration in the UDL like this:
//!
//! ```
//! # let ci = uniffi_bindgen::interface::ComponentInterface::from_webidl(r##"
//! # namespace example {};
//! dictionary Example {
//!   string name;
//!   u32 value;
//! };
//! # "##)?;
//! # Ok::<(), anyhow::Error>(())
//! ```
//!
//! Will result in a [`Record`] member with two [`Field`]s being added to the resulting
//! [`ComponentInterface`]:
//!
//! ```
//! # let ci = uniffi_bindgen::interface::ComponentInterface::from_webidl(r##"
//! # namespace example {};
//! # dictionary Example {
//! #   string name;
//! #   u32 value;
//! # };
//! # "##)?;
//! let record = ci.get_record_definition("Example").unwrap();
//! assert_eq!(record.name(), "Example");
//! assert_eq!(record.fields()[0].name(), "name");
//! assert_eq!(record.fields()[1].name(), "value");
//! # Ok::<(), anyhow::Error>(())
//! ```
use std::convert::TryFrom;

use anyhow::{bail, Result};

use super::literal::{convert_default_value, Literal};
use super::types::Type;
use super::{APIConverter, ComponentInterface};

/// Represents a "data class" style object, for passing around complex values.
///
/// In the FFI these are represented as a byte buffer, which one side explicitly
/// serializes the data into and the other serializes it out of. So I guess they're
/// kind of like "pass by clone" values.
#[derive(Debug, Clone, Default, Hash)]
pub struct Record {
    pub(super) name: String,
    pub(super) fields: Vec<Field>,
    pub(super) docs: Vec<String>,
}

impl Record {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn fields(&self) -> Vec<&Field> {
        self.fields.iter().collect()
    }

    pub fn contains_object_references(&self, ci: &ComponentInterface) -> bool {
        // *sigh* at the clone here, the relationship between a ComponentInterace
        // and its contained types could use a bit of a cleanup.
        ci.type_contains_object_references(&Type::Record(self.name.clone()))
    }

    pub fn contains_unsigned_types(&self, ci: &ComponentInterface) -> bool {
        self.fields()
            .iter()
            .any(|f| ci.type_contains_unsigned_types(&f.type_))
    }

    pub fn docs(&self) -> Vec<&str> {
        self.docs.iter().map(String::as_str).collect()
    }
}

impl APIConverter<Record> for weedle::DictionaryDefinition<'_> {
    fn convert(&self, ci: &mut ComponentInterface) -> Result<Record> {
        if self.attributes.is_some() {
            bail!("dictionary attributes are not supported yet");
        }
        if self.inheritance.is_some() {
            bail!("dictionary inheritence is not supported");
        }
        Ok(Record {
            name: self.identifier.0.to_string(),
            fields: self.members.body.convert(ci)?,
            ..Default::default()
        })
    }
}

impl APIConverter<Record> for &syn::ItemStruct {
    fn convert(&self, ci: &mut ComponentInterface) -> Result<Record> {
        let attrs = super::synner::Attributes::try_from(&self.attrs)?;
        let fields = match &self.fields {
            syn::Fields::Unit => vec![],
            syn::Fields::Unnamed(_) => bail!("Records can only have named fields"),
            syn::Fields::Named(f) => f
                .named
                .iter()
                .map(|f| f.convert(ci))
                .collect::<Result<Vec<_>>>()?,
        };
        Ok(Record {
            name: self.ident.to_string(),
            fields,
            docs: attrs.docs,
        })
    }
}

// Represents an individual field on a Record.
#[derive(Debug, Clone, Hash)]
pub struct Field {
    pub(super) name: String,
    pub(super) type_: Type,
    pub(super) required: bool,
    pub(super) default: Option<Literal>,
    pub(super) docs: Vec<String>,
}

impl Field {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn type_(&self) -> Type {
        self.type_.clone()
    }
    pub fn default_value(&self) -> Option<Literal> {
        self.default.clone()
    }
    pub fn docs(&self) -> Vec<&str> {
        self.docs.iter().map(String::as_str).collect()
    }
}

impl APIConverter<Field> for weedle::dictionary::DictionaryMember<'_> {
    fn convert(&self, ci: &mut ComponentInterface) -> Result<Field> {
        if self.attributes.is_some() {
            bail!("dictionary member attributes are not supported yet");
        }
        let type_ = ci.resolve_type_expression(&self.type_)?;
        if let Type::Object(_) = type_ {
            bail!("Objects cannot currently appear in record fields");
        }
        let default = match self.default {
            None => None,
            Some(v) => Some(convert_default_value(&v.value, &type_)?),
        };
        Ok(Field {
            name: self.identifier.0.to_string(),
            type_,
            required: self.required.is_some(),
            default,
            docs: vec![],
        })
    }
}

impl APIConverter<Field> for &syn::Field {
    fn convert(&self, ci: &mut ComponentInterface) -> Result<Field> {
        let attrs = super::synner::Attributes::try_from(&self.attrs)?;
        if !matches!(
            self.vis,
            syn::Visibility::Public(_) | syn::Visibility::Inherited
        ) {
            bail!("Variant fields must be public");
        }
        let name = match &self.ident {
            None => bail!("Variant fields must be named"),
            Some(id) => id.to_string(),
        };
        let type_ = ci.resolve_type_expression(&self.ty)?;
        if let Type::Object(_) = type_ {
            bail!("Objects cannot currently be used in enum variant data");
        }
        Ok(Field {
            name,
            type_,
            required: false,
            default: None,
            docs: attrs.docs,
        })
    }
}

#[cfg(test)]
mod test {
    use super::super::literal::Radix;
    use super::*;

    #[test]
    fn test_multiple_record_types() {
        const UDL: &str = r#"
            namespace test{};
            dictionary Empty {};
            dictionary Simple {
                u32 field;
            };
            dictionary Complex {
                string? key;
                u32 value = 0;
                required boolean spin;
            };
        "#;
        let ci = ComponentInterface::from_webidl(UDL).unwrap();
        assert_eq!(ci.iter_record_definitions().len(), 3);

        let record = ci.get_record_definition("Empty").unwrap();
        assert_eq!(record.name(), "Empty");
        assert_eq!(record.fields().len(), 0);

        let record = ci.get_record_definition("Simple").unwrap();
        assert_eq!(record.name(), "Simple");
        assert_eq!(record.fields().len(), 1);
        assert_eq!(record.fields()[0].name(), "field");
        assert_eq!(record.fields()[0].type_().canonical_name(), "u32");
        assert!(!record.fields()[0].required);
        assert!(record.fields()[0].default_value().is_none());

        let record = ci.get_record_definition("Complex").unwrap();
        assert_eq!(record.name(), "Complex");
        assert_eq!(record.fields().len(), 3);
        assert_eq!(record.fields()[0].name(), "key");
        assert_eq!(
            record.fields()[0].type_().canonical_name(),
            "Optionalstring"
        );
        assert!(!record.fields()[0].required);
        assert!(record.fields()[0].default_value().is_none());
        assert_eq!(record.fields()[1].name(), "value");
        assert_eq!(record.fields()[1].type_().canonical_name(), "u32");
        assert!(!record.fields()[1].required);
        assert!(matches!(
            record.fields()[1].default_value(),
            Some(Literal::UInt(0, Radix::Decimal, Type::UInt32))
        ));
        assert_eq!(record.fields()[2].name(), "spin");
        assert_eq!(record.fields()[2].type_().canonical_name(), "bool");
        assert!(record.fields()[2].required);
        assert!(record.fields()[2].default_value().is_none());
    }

    #[test]
    fn test_that_all_field_types_become_known() {
        const UDL: &str = r#"
            namespace test{};
            dictionary Testing {
                string? maybe_name;
                u32 value;
            };
        "#;
        let ci = ComponentInterface::from_webidl(UDL).unwrap();
        assert_eq!(ci.iter_record_definitions().len(), 1);
        let record = ci.get_record_definition("Testing").unwrap();
        assert_eq!(record.fields().len(), 2);
        assert_eq!(record.fields()[0].name(), "maybe_name");
        assert_eq!(record.fields()[1].name(), "value");

        assert_eq!(ci.iter_types().len(), 4);
        assert!(ci.iter_types().iter().any(|t| t.canonical_name() == "u32"));
        assert!(ci
            .iter_types()
            .iter()
            .any(|t| t.canonical_name() == "string"));
        assert!(ci
            .iter_types()
            .iter()
            .any(|t| t.canonical_name() == "Optionalstring"));
        assert!(ci
            .iter_types()
            .iter()
            .any(|t| t.canonical_name() == "RecordTesting"));
    }
}
