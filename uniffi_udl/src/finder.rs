/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! # Helpers for finding the named types defined in a UDL interface.
//!
//! This module provides the [`TypeFinder`] trait, an abstraction for walking
//! the weedle parse tree, looking for type definitions, and accumulating them
//! in a [`TypeCollector`].
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

use super::TypeCollector;
use crate::attributes::{InterfaceAttributes, RustKind, TypedefAttributes};
use uniffi_meta::{ObjectImpl, Type};

/// Trait to help with an early "type discovery" phase when processing the UDL.
///
/// This trait does structural matching against weedle AST nodes from a parsed
/// UDL file, looking for all the newly-defined types in the file and accumulating
/// them in the given `TypeCollector`.
pub(crate) trait TypeFinder {
    fn add_type_definitions_to(&self, types: &mut TypeCollector) -> Result<()>;
}

impl<T: TypeFinder> TypeFinder for &[T] {
    fn add_type_definitions_to(&self, types: &mut TypeCollector) -> Result<()> {
        for item in *self {
            item.add_type_definitions_to(types)?;
        }
        Ok(())
    }
}

impl TypeFinder for weedle::Definition<'_> {
    fn add_type_definitions_to(&self, types: &mut TypeCollector) -> Result<()> {
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
    fn add_type_definitions_to(&self, types: &mut TypeCollector) -> Result<()> {
        let name = self.identifier.0.to_string();
        let attrs = InterfaceAttributes::try_from(self.attributes.as_ref())?;
        // Some enum types are defined using an `interface` with a special attribute.
        if attrs.contains_enum_attr() || attrs.contains_error_attr() {
            types.add_type_definition(
                self.identifier.0,
                Type::Enum {
                    name,
                    module_path: types.module_path(),
                },
            )
        } else {
            types.add_type_definition(
                self.identifier.0,
                Type::Object {
                    name,
                    module_path: types.module_path(),
                    imp: attrs.object_impl()?,
                },
            )
        }
    }
}

impl TypeFinder for weedle::DictionaryDefinition<'_> {
    fn add_type_definitions_to(&self, types: &mut TypeCollector) -> Result<()> {
        let name = self.identifier.0.to_string();
        types.add_type_definition(
            self.identifier.0,
            Type::Record {
                name,
                module_path: types.module_path(),
            },
        )
    }
}

impl TypeFinder for weedle::EnumDefinition<'_> {
    fn add_type_definitions_to(&self, types: &mut TypeCollector) -> Result<()> {
        let name = self.identifier.0.to_string();
        // Our error types are defined using an `enum` with a special attribute.
        types.add_type_definition(
            self.identifier.0,
            Type::Enum {
                name,
                module_path: types.module_path(),
            },
        )
    }
}

impl TypeFinder for weedle::TypedefDefinition<'_> {
    fn add_type_definitions_to(&self, types: &mut TypeCollector) -> Result<()> {
        let attrs = TypedefAttributes::try_from(self.attributes.as_ref())?;
        // If we wanted simple `typedef`s, it would be as easy as:
        // > let t = types.resolve_type_expression(&self.type_)?;
        // > types.add_type_definition(name, t)
        // But we don't - `typedef`s are reserved for external types.
        if attrs.is_custom() {
            // A local type which wraps a builtin and for which we will generate an
            // `FfiConverter` implementation.
            let builtin = types.resolve_type_expression(&self.type_)?;
            types.add_type_definition(
                self.identifier.0,
                Type::Custom {
                    module_path: types.module_path(),
                    name: self.identifier.0.to_string(),
                    builtin: builtin.into(),
                },
            )
        } else {
            let typedef_type = match &self.type_.type_ {
                weedle::types::Type::Single(weedle::types::SingleType::NonAny(
                    weedle::types::NonAnyType::Identifier(weedle::types::MayBeNull {
                        type_: i,
                        ..
                    }),
                )) => i.0,
                _ => bail!("Failed to get typedef type: {:?}", self),
            };

            let module_path = types.module_path();
            let name = self.identifier.0.to_string();

            let ty = match attrs.external_tagged() {
                None => {
                    // Not external, not custom, not Rust - so we basically
                    // pretend it is Rust, thus soft-deprecating it.
                    // We use `type_`
                    match typedef_type {
                        "dictionary" | "record" | "struct" => Type::Record {
                            module_path,
                            name,
                        },
                        "enum" => Type::Enum {
                            module_path,
                            name,
                        },
                        "custom" => panic!("don't know builtin"),
                        "interface" | "impl" => Type::Object {
                            module_path,
                            name,
                            imp: ObjectImpl::Struct,
                        },
                        "trait" => Type::Object {
                            module_path,
                            name,
                            imp: ObjectImpl::Trait,
                        },
                        "callback" | "trait_with_foreign" => Type::Object {
                            module_path,
                            name,
                            imp: ObjectImpl::CallbackTrait,
                        },
                        _ => bail!("Can't work out the type - no attributes and unknown extern type '{typedef_type}'"),
                    }
                }
                Some(tagged) => {
                    // Must be either `[Rust..]` or `[Extern..]`
                    match attrs.rust_kind() {
                        Some(RustKind::Object) => Type::Object {
                            module_path,
                            name,
                            imp: ObjectImpl::Struct,
                        },
                        Some(RustKind::Trait) => Type::Object {
                            module_path,
                            name,
                            imp: ObjectImpl::Trait,
                        },
                        Some(RustKind::CallbackTrait) => Type::Object {
                            module_path,
                            name,
                            imp: ObjectImpl::CallbackTrait,
                        },
                        Some(RustKind::Record) => Type::Record { module_path, name },
                        Some(RustKind::Enum) => Type::Enum { module_path, name },
                        Some(RustKind::CallbackInterface) => {
                            Type::CallbackInterface { module_path, name }
                        }
                        // must be external
                        None => {
                            let kind = attrs.external_kind().expect("External missing kind");
                            Type::External {
                                name,
                                namespace: "".to_string(), // we don't know this yet
                                module_path: attrs.get_crate_name(),
                                kind,
                                tagged,
                            }
                        }
                    }
                }
            };
            types.add_type_definition(self.identifier.0, ty)
        }
    }
}

impl TypeFinder for weedle::CallbackInterfaceDefinition<'_> {
    fn add_type_definitions_to(&self, types: &mut TypeCollector) -> Result<()> {
        if self.attributes.is_some() {
            bail!("no callback interface attributes are currently supported");
        }
        let name = self.identifier.0.to_string();
        types.add_type_definition(
            self.identifier.0,
            Type::CallbackInterface {
                name,
                module_path: types.module_path(),
            },
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use uniffi_meta::{ExternalKind, ObjectImpl};

    // A helper to take valid UDL and a closure to check what's in it.
    fn test_a_finding<F>(udl: &str, tester: F)
    where
        F: FnOnce(TypeCollector),
    {
        let idl = weedle::parse(udl).unwrap();
        let mut types = TypeCollector::default();
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
                    matches!(types.get_type_definition("TestCallbacks").unwrap(), Type::CallbackInterface { name, .. } if name == "TestCallbacks")
                );
            },
        );

        test_a_finding(
            r#"
            dictionary TestRecord {
                u32 field;
            };
        "#,
            |types| {
                assert!(
                    matches!(types.get_type_definition("TestRecord").unwrap(), Type::Record { name, .. } if name == "TestRecord")
                );
            },
        );

        test_a_finding(
            r#"
            enum TestItems { "one", "two" };

            [Error]
            enum TestError { "ErrorOne", "ErrorTwo" };
        "#,
            |types| {
                assert!(
                    matches!(types.get_type_definition("TestItems").unwrap(), Type::Enum { name, .. } if name == "TestItems")
                );
                assert!(
                    matches!(types.get_type_definition("TestError").unwrap(), Type::Enum { name, .. } if name == "TestError")
                );
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
                    matches!(types.get_type_definition("TestObject").unwrap(), Type::Object{ name, .. } if name == "TestObject")
                );
            },
        );

        test_a_finding(
            r#"
            [External="crate-name"]
            typedef extern ExternalType;

            [ExternalInterface="crate-name"]
            typedef extern ExternalInterfaceType;

            [Custom]
            typedef string CustomType;
        "#,
            |types| {
                assert!(
                    matches!(types.get_type_definition("ExternalType").unwrap(), Type::External { name, module_path, kind: ExternalKind::DataClass, .. }
                                                                                 if name == "ExternalType" && module_path == "crate-name")
                );
                assert!(
                    matches!(types.get_type_definition("ExternalInterfaceType").unwrap(), Type::External { name, module_path, kind: ExternalKind::Interface, .. }
                                                                                 if name == "ExternalInterfaceType" && module_path == "crate-name")
                );
                assert!(
                    matches!(types.get_type_definition("CustomType").unwrap(), Type::Custom { name, builtin, ..}
                                                                                     if name == "CustomType" && *builtin == Type::String)
                );
            },
        );
    }

    #[test]
    fn test_extern_local_types() {
        // should test more, but these are already deprecated
        test_a_finding(
            r#"
            typedef interface Interface;
            typedef impl Interface2;
            typedef trait Trait;
            typedef callback Callback;

            typedef dictionary R1;
            typedef record R2;
            typedef record R3;
            typedef enum Enum;
        "#,
            |types| {
                assert!(matches!(
                    types.get_type_definition("Interface").unwrap(),
                    Type::Object { name, module_path, imp: ObjectImpl::Struct } if name == "Interface" && module_path.is_empty()));
                assert!(matches!(
                    types.get_type_definition("Interface2").unwrap(),
                    Type::Object { name, module_path, imp: ObjectImpl::Struct } if name == "Interface2" && module_path.is_empty()));
                assert!(matches!(
                    types.get_type_definition("Trait").unwrap(),
                    Type::Object { name, module_path, imp: ObjectImpl::Trait } if name == "Trait" && module_path.is_empty()));
                assert!(matches!(
                    types.get_type_definition("Callback").unwrap(),
                    Type::Object { name, module_path, imp: ObjectImpl::CallbackTrait } if name == "Callback" && module_path.is_empty()));
                assert!(matches!(
                    types.get_type_definition("R1").unwrap(),
                    Type::Record { name, module_path } if name == "R1" && module_path.is_empty()));
                assert!(matches!(
                    types.get_type_definition("R2").unwrap(),
                    Type::Record { name, module_path } if name == "R2" && module_path.is_empty()));
                assert!(matches!(
                    types.get_type_definition("R3").unwrap(),
                    Type::Record { name, module_path } if name == "R3" && module_path.is_empty()));
                assert!(matches!(
                    types.get_type_definition("Enum").unwrap(),
                    Type::Enum { name, module_path } if name == "Enum" && module_path.is_empty()));
            },
        );
    }

    #[test]
    fn test_rust_attr_types() {
        // should test more, but these are already deprecated
        test_a_finding(
            r#"
            [Rust="interface"]
            typedef extern LocalInterface;

            [Rust="dictionary"]
            typedef extern Dict;
        "#,
            |types| {
                assert!(
                    matches!(types.get_type_definition("LocalInterface").unwrap(), Type::Object { name, module_path, imp: ObjectImpl::Struct }
                                                                                if name == "LocalInterface" && module_path.is_empty())
                );
                assert!(
                    matches!(types.get_type_definition("Dict").unwrap(), Type::Record { name, module_path }
                                                                                if name == "Dict" && module_path.is_empty())
                );
            },
        );
    }

    fn get_err(udl: &str) -> String {
        let parsed = weedle::parse(udl).unwrap();
        let mut types = TypeCollector::default();
        let err = types
            .add_type_definitions_from(parsed.as_ref())
            .unwrap_err();
        err.to_string()
    }

    #[test]
    fn test_local_type_unknown_typedef() {
        let e = get_err("typedef xyz Foo;");
        assert!(e.contains("unknown extern type 'xyz'"));
    }
}
