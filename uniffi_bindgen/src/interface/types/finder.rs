/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! # Helpers for finding the named types defined in a UDL interface.
//!
//! This module provides the [`TypeFinder`] trait, an abstraction for walking
//! the weedle parse tree, looking for type definitions, and accumulating them
//! in a [`TypeUniverse`].
//!
//! The type-finding process only discovers very basic information about names
//! and their corresponding types. For example, it can discover that "Foobar"
//! names a Record, but it won't discover anything about the fields of that
//! record.
//!
//! Factoring this functionality out into a separate phase makes the subsequent
//! work of more *detailed* parsing of the UDL a lot simpler, we know how to resolve
//! names to types when building up the full interface definition.

use std::convert::TryFrom;

use anyhow::{bail, Result};

use super::super::attributes::{EnumAttributes, InterfaceAttributes};
use super::{Type, TypeUniverse};

/// Trait to help with an early "type discovery" phase when processing the UDL.
///
/// Ths trait does structural matching against weedle AST nodes from a parsed
/// UDL file, looking for all the newly-defined types in the file and accumulating
/// them in the given `TypeUniverse`.
pub(in super::super) trait TypeFinder {
    fn add_type_definitions_to(&self, types: &mut TypeUniverse) -> Result<()>;
}

impl<T: TypeFinder> TypeFinder for &[T] {
    fn add_type_definitions_to(&self, types: &mut TypeUniverse) -> Result<()> {
        for item in self.iter() {
            item.add_type_definitions_to(types)?;
        }
        Ok(())
    }
}

impl TypeFinder for weedle::Definition<'_> {
    fn add_type_definitions_to(&self, types: &mut TypeUniverse) -> Result<()> {
        match self {
            weedle::Definition::Interface(d) => d.add_type_definitions_to(types),
            weedle::Definition::Dictionary(d) => d.add_type_definitions_to(types),
            weedle::Definition::Enum(d) => d.add_type_definitions_to(types),
            weedle::Definition::Typedef(d) => d.add_type_definitions_to(types),
            weedle::Definition::CallbackInterface(d) => d.add_type_definitions_to(types),
            _ => Ok(()),
        }
    }
}

impl TypeFinder for weedle::InterfaceDefinition<'_> {
    fn add_type_definitions_to(&self, types: &mut TypeUniverse) -> Result<()> {
        let name = self.identifier.0.to_string();
        // Some enum types are defined using an `interface` with a special attribute.
        if InterfaceAttributes::try_from(self.attributes.as_ref())?.contains_enum_attr() {
            types.add_type_definition(self.identifier.0, Type::Enum(name))
        } else {
            types.add_type_definition(self.identifier.0, Type::Object(name))
        }
    }
}

impl TypeFinder for weedle::DictionaryDefinition<'_> {
    fn add_type_definitions_to(&self, types: &mut TypeUniverse) -> Result<()> {
        let name = self.identifier.0.to_string();
        types.add_type_definition(self.identifier.0, Type::Record(name))
    }
}

impl TypeFinder for weedle::EnumDefinition<'_> {
    fn add_type_definitions_to(&self, types: &mut TypeUniverse) -> Result<()> {
        let name = self.identifier.0.to_string();
        // Our error types are defined using an `enum` with a special attribute.
        if EnumAttributes::try_from(self.attributes.as_ref())?.contains_error_attr() {
            types.add_type_definition(self.identifier.0, Type::Error(name))
        } else {
            types.add_type_definition(self.identifier.0, Type::Enum(name))
        }
    }
}

impl TypeFinder for weedle::TypedefDefinition<'_> {
    fn add_type_definitions_to(&self, types: &mut TypeUniverse) -> Result<()> {
        if self.attributes.is_some() {
            bail!("no typedef attributes are currently supported");
        }
        // For now, we assume that the typedef must refer to an already-defined type, which means
        // we can look it up in the TypeUniverse. This should suffice for our needs for
        // a good long while before we consider implementing a more complex delayed resolution strategy.
        let t = types.resolve_type_expression(&self.type_)?;
        types.add_type_definition(self.identifier.0, t)
    }
}

impl TypeFinder for weedle::CallbackInterfaceDefinition<'_> {
    fn add_type_definitions_to(&self, types: &mut TypeUniverse) -> Result<()> {
        if self.attributes.is_some() {
            bail!("no typedef attributes are currently supported");
        }
        let name = self.identifier.0.to_string();
        types.add_type_definition(self.identifier.0, Type::CallbackInterface(name))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_type_finding() -> Result<()> {
        const UDL: &str = r#"
            callback interface TestCallbacks {
                string hello(u32 count);
            };

            dictionary TestRecord {
                u32 field;
            };

            enum TestItems { "one", "two" };

            [Error]
            enum TestError { "ErrorOne", "ErrorTwo" };

            interface TestObject {
                constructor();
            };

            typedef TestObject Alias;
        "#;
        let idl = weedle::parse(UDL).unwrap();
        let mut types = TypeUniverse::default();
        types.add_type_definitions_from(idl.as_ref())?;
        assert_eq!(types.iter_known_types().count(), 5);
        assert!(
            matches!(types.get_type_definition("TestCallbacks").unwrap(), Type::CallbackInterface(nm) if nm == "TestCallbacks")
        );
        assert!(
            matches!(types.get_type_definition("TestRecord").unwrap(), Type::Record(nm) if nm == "TestRecord")
        );
        assert!(
            matches!(types.get_type_definition("TestItems").unwrap(), Type::Enum(nm) if nm == "TestItems")
        );
        assert!(
            matches!(types.get_type_definition("TestError").unwrap(), Type::Error(nm) if nm == "TestError")
        );
        assert!(
            matches!(types.get_type_definition("TestObject").unwrap(), Type::Object(nm) if nm == "TestObject")
        );
        assert!(
            matches!(types.get_type_definition("Alias").unwrap(), Type::Object(nm) if nm == "TestObject")
        );
        Ok(())
    }

    #[test]
    fn test_error_on_unresolved_typedef() {
        const UDL: &str = r#"
            // Sorry, no forward declarations yet...
            typedef TestRecord Alias;

            dictionary TestRecord {
                u32 field;
            };
        "#;
        let idl = weedle::parse(UDL).unwrap();
        let mut types = TypeUniverse::default();
        let err = types.add_type_definitions_from(idl.as_ref()).unwrap_err();
        assert_eq!(err.to_string(), "unknown type reference: TestRecord");
    }
}
