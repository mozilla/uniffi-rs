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

use heck::CamelCase;
use anyhow::{bail, Result};

use super::super::attributes::{EnumAttributes, InterfaceAttributes, TypedefAttributes};
use super::{CustomTypeBindingsKind, Type, TypeUniverse};

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
        } else if InterfaceAttributes::try_from(self.attributes.as_ref())?.contains_error_attr() {
            types.add_type_definition(self.identifier.0, Type::Error(name))
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
        let name = self.identifier.0;
        let attrs = TypedefAttributes::try_from(self.attributes.as_ref())?;
        // It is simple to support aliases, but it's not clear they really
        // add value here and having such different semantics from `[Imported]`
        // ones might just add confusion.
        // If we *did*, it would be as easy as:
        // > let t = types.resolve_type_expression(&self.type_)?;
        // > types.add_type_definition(name, t)
        if attrs.is_wrapping() {
            // External hand-written type that wraps a primitive
            let wrapped_type = types.resolve_type_expression(&self.type_)?;
            types.add_type_definition(
                name,
                Type::Custom {
                    name: name.to_string(),
                    primitive: wrapped_type.clone().into(),
                    ffi_converter_name: format!("FFIConverterWrapping{}{}", name, wrapped_type.canonical_name().to_camel_case()),
                    bindings_kind: CustomTypeBindingsKind::Primitive(wrapped_type.into()),
                }
            )
        } else {
            match attrs.get_external_uniffi_crate_name() {
                // Note we don't resolve `self._type` - ideally we'd check it is the string
                // `extern`, but that seems trickier than it looks!
                Some(crate_name) => {
                    types.add_type_definition(name, Type::Custom {
                        name: name.to_string(),
                        // TODO: extract data from the external UDL file and fill in the details
                        ffi_converter_name: "TODO".into(),
                        primitive: Type::String.into(),
                        bindings_kind: CustomTypeBindingsKind::ImportedFromOtherCrate {
                            crate_name: crate_name.into(),
                        },
                    })
                }
                None => bail!("only `[ExternalUniffi=crate_name]` or `[ExternalWrapping]` typedefs are supported"),
            }
        }
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

            [ExternalUniffi="crate-name"]
            typedef extern ExternalUniffi;

            [ExternalWrapping]
            typedef string ExternalWrapping;

        "#;
        let idl = weedle::parse(UDL).unwrap();
        let mut types = TypeUniverse::default();
        types.add_type_definitions_from(idl.as_ref())?;
        assert_eq!(types.iter_known_types().count(), 8);
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
            matches!(types.get_type_definition("ExternalUniffi").unwrap(), Type::Custom { name, bindings_kind: CustomTypeBindingsKind::ImportedFromOtherCrate { crate_name }, .. }
                                                                           if name == "ExternalUniffi" && crate_name == "crate-name")
        );
        assert!(
            matches!(types.get_type_definition("ExternalWrapping").unwrap(), Type::Custom { name, bindings_kind: CustomTypeBindingsKind::Primitive(primitive), ..}
                                                                             if name == "ExternalWrapping" && primitive == Box::new(Type::String))
        );
        // Our `typedef string External` has caused `String` to also be known.
        assert!(types.all_known_types.contains(&Type::String));

        Ok(())
    }

    fn get_err(udl: &str) -> String {
        let parsed = weedle::parse(udl).unwrap();
        let mut types = TypeUniverse::default();
        let err = types
            .add_type_definitions_from(parsed.as_ref())
            .unwrap_err();
        err.to_string()
    }

    #[test]
    fn test_typedef_error_on_no_attr() {
        // Sorry, still working out what we want for non-imported typedefs..
        assert_eq!(
            get_err("typedef string Custom;"),
            "only `[ExternalUniffi=crate_name]` or `[ExternalWrapping]` typedefs are supported"
        );
    }

    #[test]
    fn test_typedef_uniffi_must_have_crate() {
        assert_eq!(
            get_err("[ExternalUniffi]typedef extern Custom;"),
            "ExtendedAttributeNoArgs not supported: \"ExternalUniffi\""
        );
    }

    #[test]
    fn test_typedef_wrapped_takes_no_value() {
        assert_eq!(
            get_err("[ExternalWrapped=something]typedef string Custom;"),
            "Attribute identity Identifier not supported: \"ExternalWrapped\""
        );
    }

    #[test]
    fn test_typedef_extern_unknown_type() {
        assert_eq!(
            get_err("[ExternalWrapping]typedef UnknownType Custom;"),
            "unknown type reference: UnknownType"
        );
    }
}
