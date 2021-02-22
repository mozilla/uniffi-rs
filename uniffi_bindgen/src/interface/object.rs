/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! # Object definitions for a `ComponentInterface`.
//!
//! This module converts "interface" definitions from UDL into [`Object`] structures
//! that can be added to a `ComponentInterface`, which are the main way we define stateful
//! objects with behaviour for a UniFFI Rust Component. An [`Object`] is an opaque handle
//! to some state on which methods can be invoked.
//!
//! (The terminology mismatch between "interface" and "object" is a historical artifact of
//! this tool prior to committing to WebIDL syntax).
//!
//! A declaration in the UDL like this:
//!
//! ```
//! # let ci = uniffi_bindgen::interface::ComponentInterface::from_webidl(r##"
//! # namespace example {};
//! interface Example {
//!   constructor(string? name);
//!   string my_name();
//! };
//! # "##)?;
//! # Ok::<(), anyhow::Error>(())
//! ```
//!
//! Will result in an [`Object`] member with one [`Constructor`] and one [`Method`] being added
//! to the resulting [`ComponentInterface`]:
//!
//! ```
//! # let ci = uniffi_bindgen::interface::ComponentInterface::from_webidl(r##"
//! # namespace example {};
//! # interface Example {
//! #   constructor(string? name);
//! #   string my_name();
//! # };
//! # "##)?;
//! let obj = ci.get_object_definition("Example").unwrap();
//! assert_eq!(obj.name(), "Example");
//! assert_eq!(obj.constructors().len(), 1);
//! assert_eq!(obj.constructors()[0].arguments()[0].name(), "name");
//! assert_eq!(obj.methods().len(),1 );
//! assert_eq!(obj.methods()[0].name(), "my_name");
//! # Ok::<(), anyhow::Error>(())
//! ```

use std::collections::HashSet;
use std::convert::TryFrom;
use std::hash::{Hash, Hasher};

use anyhow::{bail, Result};

use super::attributes::{ConstructorAttributes, InterfaceAttributes, MethodAttributes};
use super::ffi::{FFIArgument, FFIFunction, FFIType};
use super::function::Argument;
use super::types::Type;
use super::{APIConverter, ComponentInterface};

/// An "object" is an opaque type that can be instantiated and passed around by reference,
/// have methods called on it, and so on - basically your classic Object Oriented Programming
/// type of deal, except without elaborate inheritence hierarchies.
///
/// In UDL these correspond to the `interface` keyword.
///
/// At the FFI layer, objects are represented by an opaque integer handle and a set of functions
/// a common prefix. The object's constuctors are functions that return new objects by handle,
/// and its methods are functions that take a handle as first argument. The foreign language
/// binding code is expected to stitch these functions back together into an appropriate class
/// definition (or that language's equivalent thereof).
///
/// TODO:
///  - maybe "Class" would be a better name than "Object" here?
#[derive(Debug, Clone)]
pub struct Object {
    pub(super) name: String,
    pub(super) constructors: Vec<Constructor>,
    pub(super) methods: Vec<Method>,
    pub(super) ffi_func_free: FFIFunction,
    pub(super) threadsafe: bool,
}

impl Object {
    fn new(name: String) -> Object {
        Object {
            name,
            constructors: Default::default(),
            methods: Default::default(),
            ffi_func_free: Default::default(),
            threadsafe: false,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn constructors(&self) -> Vec<&Constructor> {
        self.constructors.iter().collect()
    }

    pub fn primary_constructor(&self) -> Option<&Constructor> {
        self.constructors
            .iter()
            .find(|cons| cons.is_primary_constructor())
    }

    pub fn alternate_constructors(&self) -> Vec<&Constructor> {
        self.constructors
            .iter()
            .filter(|cons| !cons.is_primary_constructor())
            .collect()
    }

    pub fn methods(&self) -> Vec<&Method> {
        self.methods.iter().collect()
    }

    pub fn ffi_object_free(&self) -> &FFIFunction {
        &self.ffi_func_free
    }

    pub fn threadsafe(&self) -> bool {
        self.threadsafe
    }

    pub fn derive_ffi_funcs(&mut self, ci_prefix: &str) -> Result<()> {
        self.ffi_func_free.name = format!("ffi_{}_{}_object_free", ci_prefix, self.name);
        self.ffi_func_free.arguments = vec![FFIArgument {
            name: "handle".to_string(),
            type_: FFIType::UInt64,
        }];
        self.ffi_func_free.return_type = None;
        for cons in self.constructors.iter_mut() {
            cons.derive_ffi_func(ci_prefix, &self.name)?
        }
        for meth in self.methods.iter_mut() {
            meth.derive_ffi_func(ci_prefix, &self.name)?
        }
        Ok(())
    }
}

impl Hash for Object {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // We don't include the FFIFunc in the hash calculation, because:
        //  - it is entirely determined by the other fields,
        //    so excluding it is safe.
        //  - its `name` property includes a checksum derived from  the very
        //    hash value we're trying to calculate here, so excluding it
        //    avoids a weird circular depenendency in the calculation.
        self.name.hash(state);
        self.constructors.hash(state);
        self.methods.hash(state);
    }
}

impl APIConverter<Object> for weedle::InterfaceDefinition<'_> {
    fn convert(&self, ci: &mut ComponentInterface) -> Result<Object> {
        if self.inheritance.is_some() {
            bail!("interface inheritence is not supported");
        }
        let mut object = Object::new(self.identifier.0.to_string());
        let attributes = match &self.attributes {
            Some(attrs) => InterfaceAttributes::try_from(attrs)?,
            None => Default::default(),
        };
        object.threadsafe = attributes.threadsafe();
        // Convert each member into a constructor or method, guarding against duplicate names.
        let mut member_names = HashSet::new();
        for member in &self.members.body {
            match member {
                weedle::interface::InterfaceMember::Constructor(t) => {
                    let cons: Constructor = t.convert(ci)?;
                    if !member_names.insert(cons.name.clone()) {
                        bail!("Duplicate interface member name: \"{}\"", cons.name())
                    }
                    object.constructors.push(cons);
                }
                weedle::interface::InterfaceMember::Operation(t) => {
                    let mut method: Method = t.convert(ci)?;
                    if !member_names.insert(method.name.clone()) {
                        bail!("Duplicate interface member name: \"{}\"", method.name())
                    }
                    method.object_name.push_str(object.name.as_str());
                    object.methods.push(method);
                }
                _ => bail!("no support for interface member type {:?} yet", member),
            }
        }
        // Everyone gets a primary constructor, even if not declared explicitly.
        if object.primary_constructor().is_none() {
            object.constructors.push(Default::default());
        }
        Ok(object)
    }
}

// Represents a constructor for an object type.
//
// In the FFI, this will be a function that returns a handle for an instance
// of the corresponding object type.
#[derive(Debug, Clone)]
pub struct Constructor {
    pub(super) name: String,
    pub(super) arguments: Vec<Argument>,
    pub(super) ffi_func: FFIFunction,
    pub(super) attributes: ConstructorAttributes,
}

impl Constructor {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn arguments(&self) -> Vec<&Argument> {
        self.arguments.iter().collect()
    }

    pub fn ffi_func(&self) -> &FFIFunction {
        &self.ffi_func
    }

    pub fn throws(&self) -> Option<&str> {
        self.attributes.get_throws_err()
    }

    fn derive_ffi_func(&mut self, ci_prefix: &str, obj_prefix: &str) -> Result<()> {
        self.ffi_func.name.push_str(ci_prefix);
        self.ffi_func.name.push_str("_");
        self.ffi_func.name.push_str(obj_prefix);
        self.ffi_func.name.push_str("_");
        self.ffi_func.name.push_str(&self.name);
        self.ffi_func.arguments = self.arguments.iter().map(Into::into).collect();
        self.ffi_func.return_type = Some(FFIType::UInt64);
        Ok(())
    }

    fn is_primary_constructor(&self) -> bool {
        self.name == "new"
    }
}

impl Hash for Constructor {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // We don't include the FFIFunc in the hash calculation, because:
        //  - it is entirely determined by the other fields,
        //    so excluding it is safe.
        //  - its `name` property includes a checksum derived from  the very
        //    hash value we're trying to calculate here, so excluding it
        //    avoids a weird circular depenendency in the calculation.
        self.name.hash(state);
        self.arguments.hash(state);
        self.attributes.hash(state);
    }
}

impl Default for Constructor {
    fn default() -> Self {
        Constructor {
            name: String::from("new"),
            arguments: Vec::new(),
            ffi_func: Default::default(),
            attributes: Default::default(),
        }
    }
}

impl APIConverter<Constructor> for weedle::interface::ConstructorInterfaceMember<'_> {
    fn convert(&self, ci: &mut ComponentInterface) -> Result<Constructor> {
        let attributes = match &self.attributes {
            Some(attr) => ConstructorAttributes::try_from(attr)?,
            None => Default::default(),
        };
        Ok(Constructor {
            name: String::from(attributes.get_name().unwrap_or("new")),
            arguments: self.args.body.list.convert(ci)?,
            ffi_func: Default::default(),
            attributes,
        })
    }
}

// Represents an instance method for an object type.
//
// The in FFI, this will be a function whose first argument is a handle for an
// instance of the corresponding object type.
#[derive(Debug, Clone)]
pub struct Method {
    pub(super) name: String,
    pub(super) object_name: String,
    pub(super) return_type: Option<Type>,
    pub(super) arguments: Vec<Argument>,
    pub(super) ffi_func: FFIFunction,
    pub(super) attributes: MethodAttributes,
}

impl Method {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn arguments(&self) -> Vec<&Argument> {
        self.arguments.iter().collect()
    }

    pub fn return_type(&self) -> Option<&Type> {
        self.return_type.as_ref()
    }

    pub fn ffi_func(&self) -> &FFIFunction {
        &self.ffi_func
    }

    pub fn first_argument(&self) -> Argument {
        Argument {
            name: "handle".to_string(),
            // TODO: ideally we'd get this via `ci.resolve_type_expression` so that it
            // is contained in the proper `TypeUniverse`, but this works for now.
            type_: Type::Object(self.object_name.clone()),
            by_ref: false,
            optional: false,
            default: None,
        }
    }

    pub fn throws(&self) -> Option<&str> {
        self.attributes.get_throws_err()
    }

    pub fn derive_ffi_func(&mut self, ci_prefix: &str, obj_prefix: &str) -> Result<()> {
        self.ffi_func.name.push_str(ci_prefix);
        self.ffi_func.name.push_str("_");
        self.ffi_func.name.push_str(obj_prefix);
        self.ffi_func.name.push_str("_");
        self.ffi_func.name.push_str(&self.name);
        self.ffi_func.arguments = vec![self.first_argument()]
            .iter()
            .chain(self.arguments.iter())
            .map(Into::into)
            .collect();
        self.ffi_func.return_type = self.return_type.as_ref().map(Into::into);
        Ok(())
    }
}

impl Hash for Method {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // We don't include the FFIFunc in the hash calculation, because:
        //  - it is entirely determined by the other fields,
        //    so excluding it is safe.
        //  - its `name` property includes a checksum derived from  the very
        //    hash value we're trying to calculate here, so excluding it
        //    avoids a weird circular depenendency in the calculation.
        self.name.hash(state);
        self.object_name.hash(state);
        self.arguments.hash(state);
        self.return_type.hash(state);
        self.attributes.hash(state);
    }
}

impl APIConverter<Method> for weedle::interface::OperationInterfaceMember<'_> {
    fn convert(&self, ci: &mut ComponentInterface) -> Result<Method> {
        if self.special.is_some() {
            bail!("special operations not supported");
        }
        if self.modifier.is_some() {
            bail!("method modifiers are not supported")
        }
        let return_type = ci.resolve_return_type_expression(&self.return_type)?;
        if let Some(Type::Object(_)) = return_type {
            bail!("Objects cannot currently be returned from functions");
        }
        Ok(Method {
            name: match self.identifier {
                None => bail!("anonymous methods are not supported {:?}", self),
                Some(id) => {
                    let name = id.0.to_string();
                    if name == "new" {
                        bail!("the method name \"new\" is reserved for the default constructor");
                    }
                    name
                }
            },
            // We don't know the name of the containing `Object` at this point, fill it in later.
            object_name: Default::default(),
            arguments: self.args.body.list.convert(ci)?,
            return_type,
            ffi_func: Default::default(),
            attributes: MethodAttributes::try_from(self.attributes.as_ref())?,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_that_all_argument_and_return_types_become_known() -> Result<()> {
        const UDL: &str = r#"
            namespace test{};
            interface Testing {
                constructor(string? name, u16 age);
                sequence<u32> code_points_of_name();
            };
        "#;
        let ci = ComponentInterface::from_webidl(UDL).unwrap();
        assert_eq!(ci.iter_object_definitions().len(), 1);
        ci.get_object_definition("Testing").unwrap();

        assert_eq!(ci.iter_types().len(), 6);
        assert!(ci
            .iter_types()
            .iter()
            .find(|t| t.canonical_name() == "u16")
            .is_some());
        assert!(ci
            .iter_types()
            .iter()
            .find(|t| t.canonical_name() == "u32")
            .is_some());
        assert!(ci
            .iter_types()
            .iter()
            .find(|t| t.canonical_name() == "Sequenceu32")
            .is_some());
        assert!(ci
            .iter_types()
            .iter()
            .find(|t| t.canonical_name() == "string")
            .is_some());
        assert!(ci
            .iter_types()
            .iter()
            .find(|t| t.canonical_name() == "Optionalstring")
            .is_some());
        assert!(ci
            .iter_types()
            .iter()
            .find(|t| t.canonical_name() == "ObjectTesting")
            .is_some());

        Ok(())
    }

    #[test]
    fn test_alternate_constructors() -> Result<()> {
        const UDL: &str = r#"
            namespace test{};
            interface Testing {
                constructor();
                [Name=new_with_u32]
                constructor(u32 v);
            };
        "#;
        let ci = ComponentInterface::from_webidl(UDL).unwrap();
        assert_eq!(ci.iter_object_definitions().len(), 1);

        let obj = ci.get_object_definition("Testing").unwrap();
        assert!(obj.primary_constructor().is_some());
        assert_eq!(obj.alternate_constructors().len(), 1);
        assert_eq!(obj.methods().len(), 0);

        let cons = obj.primary_constructor().unwrap();
        assert_eq!(cons.name(), "new");
        assert_eq!(cons.arguments.len(), 0);
        assert_eq!(cons.ffi_func.arguments.len(), 0);

        let cons = obj.alternate_constructors()[0];
        assert_eq!(cons.name(), "new_with_u32");
        assert_eq!(cons.arguments.len(), 1);
        assert_eq!(cons.ffi_func.arguments.len(), 1);

        Ok(())
    }

    #[test]
    fn test_the_name_new_identifies_the_primary_constructor() -> Result<()> {
        const UDL: &str = r#"
            namespace test{};
            interface Testing {
                [Name=newish]
                constructor();
                [Name=new]
                constructor(u32 v);
            };
        "#;
        let ci = ComponentInterface::from_webidl(UDL).unwrap();
        assert_eq!(ci.iter_object_definitions().len(), 1);

        let obj = ci.get_object_definition("Testing").unwrap();
        assert!(obj.primary_constructor().is_some());
        assert_eq!(obj.alternate_constructors().len(), 1);
        assert_eq!(obj.methods().len(), 0);

        let cons = obj.primary_constructor().unwrap();
        assert_eq!(cons.name(), "new");
        assert_eq!(cons.arguments.len(), 1);

        let cons = obj.alternate_constructors()[0];
        assert_eq!(cons.name(), "newish");
        assert_eq!(cons.arguments.len(), 0);
        assert_eq!(cons.ffi_func.arguments.len(), 0);

        Ok(())
    }

    #[test]
    fn test_the_name_new_is_reserved_for_constructors() {
        const UDL: &str = r#"
            namespace test{};
            interface Testing {
                constructor();
                void new(u32 v);
            };
        "#;
        let err = ComponentInterface::from_webidl(UDL).unwrap_err();
        assert_eq!(
            err.to_string(),
            "the method name \"new\" is reserved for the default constructor"
        );
    }

    #[test]
    fn test_duplicate_primary_constructors_not_allowed() {
        const UDL: &str = r#"
            namespace test{};
            interface Testing {
                constructor();
                constructor(u32 v);
            };
        "#;
        let err = ComponentInterface::from_webidl(UDL).unwrap_err();
        assert_eq!(err.to_string(), "Duplicate interface member name: \"new\"");

        const UDL2: &str = r#"
            namespace test{};
            interface Testing {
                constructor();
                [Name=new]
                constructor(u32 v);
            };
        "#;
        let err = ComponentInterface::from_webidl(UDL2).unwrap_err();
        assert_eq!(err.to_string(), "Duplicate interface member name: \"new\"");
    }
}
