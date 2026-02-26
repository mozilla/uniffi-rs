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
//! # "##, "crate_name")?;
//! # Ok::<(), anyhow::Error>(())
//! ```
//!
//! Will result in a [`Record`] member with two [`Field`]s being added to the resulting
//! [`crate::ComponentInterface`]:
//!
//! ```
//! # let ci = uniffi_bindgen::interface::ComponentInterface::from_webidl(r##"
//! # namespace example {};
//! # dictionary Example {
//! #   string name;
//! #   u32 value;
//! # };
//! # "##, "crate_name")?;
//! let record = ci.get_record_definition("Example").unwrap();
//! assert_eq!(record.name(), "Example");
//! assert_eq!(record.fields()[0].name(), "name");
//! assert_eq!(record.fields()[1].name(), "value");
//! # Ok::<(), anyhow::Error>(())
//! ```

use anyhow::Result;
use uniffi_meta::Checksum;

use super::function::Callable;
use super::{
    AsType, Constructor, DefaultValue, FfiFunction, Method, Type, TypeIterator, UniffiTrait,
    UniffiTraitMethods,
};
use uniffi_meta::ObjectTraitImplMetadata;

/// Represents a "data class" style object, for passing around complex values.
///
/// In the FFI these are represented as a byte buffer, which one side explicitly
/// serializes the data into and the other serializes it out of. So I guess they're
/// kind of like "pass by clone" values.
#[derive(Debug, Clone, Checksum)]
pub struct Record {
    pub(super) name: String,
    pub(super) module_path: String,
    pub(super) remote: bool,
    pub(super) fields: Vec<Field>,
    pub(super) constructors: Vec<Constructor>,
    pub(super) methods: Vec<Method>,
    pub uniffi_traits: Vec<UniffiTrait>,
    pub(super) trait_impls: Vec<ObjectTraitImplMetadata>,
    #[checksum_ignore]
    pub(super) docstring: Option<String>,
}

impl Record {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn rename(&mut self, name: String) {
        self.name = name;
    }

    pub fn remote(&self) -> bool {
        self.remote
    }

    pub fn fields(&self) -> &[Field] {
        &self.fields
    }

    pub fn constructors(&self) -> &[Constructor] {
        &self.constructors
    }

    pub fn methods(&self) -> &[Method] {
        &self.methods
    }

    pub fn docstring(&self) -> Option<&str> {
        self.docstring.as_deref()
    }

    pub fn iter_types(&self) -> TypeIterator<'_> {
        Box::new(
            self.fields
                .iter()
                .flat_map(Field::iter_types)
                .chain(self.constructors.iter().flat_map(Constructor::iter_types))
                .chain(self.methods.iter().flat_map(Method::iter_types)),
        )
    }

    pub fn has_fields(&self) -> bool {
        !self.fields.is_empty()
    }

    pub fn uniffi_trait_methods(&self) -> UniffiTraitMethods {
        UniffiTraitMethods::new(&self.uniffi_traits)
    }

    pub fn add_uniffi_trait(&mut self, t: UniffiTrait) {
        self.uniffi_traits.push(t);
    }

    pub fn trait_impls(&self) -> Vec<&ObjectTraitImplMetadata> {
        self.trait_impls.iter().collect()
    }

    pub fn trait_impls_mut(&mut self) -> &mut Vec<ObjectTraitImplMetadata> {
        &mut self.trait_impls
    }

    pub fn derive_ffi_funcs(&mut self) -> Result<()> {
        for c in self.constructors.iter_mut() {
            c.derive_ffi_func();
        }
        for m in self.methods.iter_mut() {
            m.derive_ffi_func()?;
        }
        for ut in self.uniffi_traits.iter_mut() {
            ut.derive_ffi_func()?;
        }
        Ok(())
    }

    pub fn iter_ffi_function_definitions(&self) -> impl Iterator<Item = &FfiFunction> {
        self.constructors
            .iter()
            .map(|f| &f.ffi_func)
            .chain(self.methods.iter().map(|f| &f.ffi_func))
            .chain(
                self.uniffi_traits
                    .iter()
                    .flat_map(|ut| match ut {
                        UniffiTrait::Display { fmt: m }
                        | UniffiTrait::Debug { fmt: m }
                        | UniffiTrait::Hash { hash: m }
                        | UniffiTrait::Ord { cmp: m } => vec![m],
                        UniffiTrait::Eq { eq, ne } => vec![eq, ne],
                    })
                    .map(|m| &m.ffi_func),
            )
    }
}

impl AsType for Record {
    fn as_type(&self) -> Type {
        Type::Record {
            name: self.name.clone(),
            module_path: self.module_path.clone(),
        }
    }
}

impl TryFrom<uniffi_meta::RecordMetadata> for Record {
    type Error = anyhow::Error;

    fn try_from(meta: uniffi_meta::RecordMetadata) -> Result<Self> {
        Ok(Self {
            name: meta.name,
            module_path: meta.module_path,
            remote: meta.remote,
            fields: meta
                .fields
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_>>()?,
            constructors: vec![],
            methods: vec![],
            uniffi_traits: vec![],
            trait_impls: vec![],
            docstring: meta.docstring.clone(),
        })
    }
}

// Represents an individual field on a Record.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Checksum)]
pub struct Field {
    pub(super) name: String,
    pub(super) type_: Type,
    pub(super) default: Option<DefaultValue>,
    #[checksum_ignore]
    pub(super) docstring: Option<String>,
}

impl Field {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn rename(&mut self, name: String) {
        self.name = name;
    }

    pub fn default_value(&self) -> Option<&DefaultValue> {
        self.default.as_ref()
    }

    pub fn docstring(&self) -> Option<&str> {
        self.docstring.as_deref()
    }

    pub fn iter_types(&self) -> TypeIterator<'_> {
        self.type_.iter_types()
    }
}

impl AsType for Field {
    fn as_type(&self) -> Type {
        self.type_.clone()
    }
}

impl TryFrom<uniffi_meta::FieldMetadata> for Field {
    type Error = anyhow::Error;

    fn try_from(meta: uniffi_meta::FieldMetadata) -> Result<Self> {
        let name = meta.name;
        let type_ = meta.ty;
        let default = meta.default;
        Ok(Self {
            name,
            type_,
            default,
            docstring: meta.docstring.clone(),
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::interface::{ComponentInterface, Literal};
    use uniffi_meta::Radix;

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
        let ci = ComponentInterface::from_webidl(UDL, "crate_name").unwrap();
        assert_eq!(ci.record_definitions().len(), 3);

        let record = ci.get_record_definition("Empty").unwrap();
        assert_eq!(record.name(), "Empty");
        assert_eq!(record.fields().len(), 0);

        let record = ci.get_record_definition("Simple").unwrap();
        assert_eq!(record.name(), "Simple");
        assert_eq!(record.fields().len(), 1);
        assert_eq!(record.fields()[0].name(), "field");
        assert_eq!(record.fields()[0].as_type(), Type::UInt32);
        assert!(record.fields()[0].default_value().is_none());

        let record = ci.get_record_definition("Complex").unwrap();
        assert_eq!(record.name(), "Complex");
        assert_eq!(record.fields().len(), 3);
        assert_eq!(record.fields()[0].name(), "key");
        assert_eq!(
            record.fields()[0].as_type(),
            Type::Optional {
                inner_type: Box::new(Type::String)
            },
        );
        assert!(record.fields()[0].default_value().is_none());
        assert_eq!(record.fields()[1].name(), "value");
        assert_eq!(record.fields()[1].as_type(), Type::UInt32);
        assert!(matches!(
            record.fields()[1].default_value(),
            Some(DefaultValue::Literal(Literal::UInt(
                0,
                Radix::Decimal,
                Type::UInt32
            )))
        ));
        assert_eq!(record.fields()[2].name(), "spin");
        assert_eq!(record.fields()[2].as_type(), Type::Boolean);
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
        let ci = ComponentInterface::from_webidl(UDL, "crate_name").unwrap();
        assert_eq!(ci.record_definitions().len(), 1);
        let record = ci.get_record_definition("Testing").unwrap();
        assert_eq!(record.fields().len(), 2);
        assert_eq!(record.fields()[0].name(), "maybe_name");
        assert_eq!(record.fields()[1].name(), "value");

        assert_eq!(ci.iter_local_types().count(), 4);
        assert!(ci.iter_local_types().any(|t| t == &Type::UInt32));
        assert!(ci.iter_local_types().any(|t| t == &Type::String));
        assert!(ci.iter_local_types().any(|t| t
            == &Type::Optional {
                inner_type: Box::new(Type::String)
            }));
        assert!(ci
            .iter_local_types()
            .any(|t| matches!(t, Type::Record { name, .. } if name == "Testing")));
    }

    #[test]
    fn test_docstring_record() {
        const UDL: &str = r#"
            namespace test{};
            /// informative docstring
            dictionary Testing { };
        "#;
        let ci = ComponentInterface::from_webidl(UDL, "crate_name").unwrap();
        assert_eq!(
            ci.get_record_definition("Testing")
                .unwrap()
                .docstring()
                .unwrap(),
            "informative docstring"
        );
    }

    #[test]
    fn test_docstring_record_field() {
        const UDL: &str = r#"
            namespace test{};
            dictionary Testing {
                /// informative docstring
                i32 testing;
            };
        "#;
        let ci = ComponentInterface::from_webidl(UDL, "crate_name").unwrap();
        assert_eq!(
            ci.get_record_definition("Testing").unwrap().fields()[0]
                .docstring()
                .unwrap(),
            "informative docstring"
        );
    }
}
