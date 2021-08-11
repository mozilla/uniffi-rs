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

use super::super::attributes::{DictionaryAttributes, EnumAttributes, InterfaceAttributes};
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
        let ia = InterfaceAttributes::try_from(self.attributes.as_ref())?;
        if ia.contains_enum_attr() {
            if ia.is_exported() {
                types.mark_as_exported(&name)
            }
            types.add_type_definition(self.identifier.0, Type::Enum(name))
        } else if ia.contains_error_attr() {
            if ia.is_exported() {
                types.mark_as_exported(&name)
            }
            types.add_type_definition(self.identifier.0, Type::Error(name))
        } else {
            // `[Exported]` on interfaces is a bit hacky - we support the attribute on an
            // `Interface` because sometimes they are actually enums or errors, as handled above.
            // But we don't allow "real" interfaces to be exported because it's tricky to generate
            // the code and there's no good reason - the original can just be used directly.
            if ia.is_exported() {
                bail!("[Export] on interfaces isn't supported");
            }
            types.add_type_definition(self.identifier.0, Type::Object(name))
        }
    }
}

impl TypeFinder for weedle::DictionaryDefinition<'_> {
    fn add_type_definitions_to(&self, types: &mut TypeUniverse) -> Result<()> {
        let name = self.identifier.0.to_string();
        if DictionaryAttributes::try_from(self.attributes.as_ref())?.is_exported() {
            types.mark_as_exported(&name);
        }
        types.add_type_definition(self.identifier.0, Type::Record(name))
    }
}

impl TypeFinder for weedle::EnumDefinition<'_> {
    fn add_type_definitions_to(&self, types: &mut TypeUniverse) -> Result<()> {
        let name = self.identifier.0.to_string();
        // Our error types are defined using an `enum` with a special attribute.
        let ea = EnumAttributes::try_from(self.attributes.as_ref())?;
        if ea.is_exported() {
            types.mark_as_exported(&name);
        }
        if ea.contains_error_attr() {
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

    // A helper to take valid UDL and a closure to check what's in it.
    fn test_a_finding<F>(udl: &str, tester: F)
    where
        F: FnOnce(TypeUniverse),
    {
        let idl = weedle::parse(udl).unwrap();
        let mut types = TypeUniverse::default();
        types.add_type_definitions_from(idl.as_ref()).unwrap();
        tester(types);
    }

    #[test]
    fn test_type_finding() {
        test_a_finding(
            r#"
            callback interface TestCallbacks {
                string hello(u32 count);
            };
        "#,
            |types| {
                assert!(
                    matches!(types.get_type_definition("TestCallbacks").unwrap(), Type::CallbackInterface(nm) if nm == "TestCallbacks")
                );
            },
        );

        test_a_finding(
            r#"
            dictionary TestRecord {
                u32 field;
            };

            [Export]
            dictionary ExportedTestRecord {
                u32 field;
            };
        "#,
            |types| {
                assert!(
                    matches!(types.get_type_definition("TestRecord").unwrap(), Type::Record(nm) if nm == "TestRecord")
                );
                assert!(!types.is_exported("TestRecord"));
                assert!(
                    matches!(types.get_type_definition("ExportedTestRecord").unwrap(), Type::Record(nm) if nm == "ExportedTestRecord")
                );
                assert!(types.is_exported("ExportedTestRecord"));
            },
        );

        test_a_finding(
            r#"
            enum TestItems { "one", "two" };

            [Export]
            enum ExportedTestItems { "one", "two" };

            [Error]
            enum TestError { "ErrorOne", "ErrorTwo" };

            [Error, Export]
            enum ExportedTestError { "ErrorOne", "ErrorTwo" };

        "#,
            |types| {
                assert!(
                    matches!(types.get_type_definition("TestItems").unwrap(), Type::Enum(nm) if nm == "TestItems")
                );
                assert!(!types.is_exported("TestItems"));

                assert!(
                    matches!(types.get_type_definition("ExportedTestItems").unwrap(), Type::Enum(nm) if nm == "ExportedTestItems")
                );
                assert!(types.is_exported("ExportedTestItems"));

                assert!(
                    matches!(types.get_type_definition("TestError").unwrap(), Type::Error(nm) if nm == "TestError")
                );
                assert!(!types.is_exported("TestError"));

                assert!(
                    matches!(types.get_type_definition("ExportedTestError").unwrap(), Type::Error(nm) if nm == "ExportedTestError")
                );
                assert!(types.is_exported("ExportedTestError"));
            },
        );

        test_a_finding(
            r#"
            interface TestObject {
                constructor();
            };
        "#,
            |types| {
                assert!(
                    matches!(types.get_type_definition("TestObject").unwrap(), Type::Object(nm) if nm == "TestObject")
                );
            },
        );

        test_a_finding(
            r#"
            interface TestObject {};
            typedef TestObject Alias;
        "#,
            |types| {
                assert!(
                    matches!(types.get_type_definition("TestObject").unwrap(), Type::Object(nm) if nm == "TestObject")
                );
                assert!(
                    matches!(types.get_type_definition("Alias").unwrap(), Type::Object(nm) if nm == "TestObject")
                );
            },
        );
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
