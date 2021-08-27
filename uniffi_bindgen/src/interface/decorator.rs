/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! # Decorator object definitions for a `ComponentInterface`.
//!
//! This module converts "interface" definitions from UDL into [`DecoratorObject`] structures
//! that can be added to a `ComponentInterface`.
//!
//! A [`DecoratorObject`] is a collection of methods defined by application code.
//!
//! A declaration in the UDL like this:
//!
//! ```
//! # let ci = uniffi_bindgen::interface::ComponentInterface::from_webidl(r##"
//! # namespace example {};
//! [Decorator]
//! interface ExampleDecorator {
//!   void async_dispatch();
//! };
//! [Decorator=ExampleDecorator]
//! interface Example {
//!   [CallsWith=async_dispatch]
//!   void long_running_method();
//! };
//!
//! # "##)?;
//! # Ok::<(), anyhow::Error>(())
//! ```
//!
//! Will result in an [`DecoratorObject`] with one [`DecoratorMethod`] and a corresponding [`Object`](super::Object)
//! which uses that decorator object.
//!
//! ```
//! # let ci = uniffi_bindgen::interface::ComponentInterface::from_webidl(r##"
//! # namespace example {};
//! # [Decorator]
//! # interface ExampleDecorator {
//! #   void async_dispatch();
//! # };
//! # "##)?;
//! let obj = ci.get_decorator_definition("ExampleDecorator").unwrap();
//! assert_eq!(obj.name(), "ExampleDecorator");
//! assert_eq!(obj.methods().len(),1 );
//! assert_eq!(obj.methods()[0].name(), "async_dispatch");
//! # Ok::<(), anyhow::Error>(())
//! ```

use crate::interface::Type;
use std::collections::HashSet;
use std::convert::TryFrom;
use std::hash::{Hash, Hasher};

use anyhow::{bail, Result};

use super::attributes::MethodAttributes;
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
pub struct DecoratorObject {
    pub(super) name: String,
    pub(super) methods: Vec<DecoratorMethod>,
}

impl DecoratorObject {
    fn new(name: String) -> Self {
        Self {
            name,
            methods: Default::default(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn type_(&self) -> Type {
        Type::DecoratorObject(self.name.clone())
    }

    pub fn methods(&self) -> Vec<&DecoratorMethod> {
        self.methods.iter().collect()
    }

    pub fn find_method(&self, nm: &str) -> Option<&DecoratorMethod> {
        self.methods.iter().find(|m| m.name == nm)
    }
}

impl Hash for DecoratorObject {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.methods.hash(state);
    }
}

impl APIConverter<DecoratorObject> for weedle::InterfaceDefinition<'_> {
    fn convert(&self, ci: &mut ComponentInterface) -> Result<DecoratorObject> {
        if self.inheritance.is_some() {
            bail!("interface inheritence is not supported");
        }
        let mut decorator = DecoratorObject::new(self.identifier.0.to_string());
        // Convert each member into a constructor or method, guarding against duplicate names.
        let mut member_names = HashSet::new();
        for member in &self.members.body {
            match member {
                weedle::interface::InterfaceMember::Operation(t) => {
                    let mut method: DecoratorMethod = t.convert(ci)?;
                    if !member_names.insert(method.name.clone()) {
                        bail!("Duplicate interface member name: \"{}\"", method.name())
                    }
                    method.object_name = decorator.name.clone();
                    decorator.methods.push(method);
                }
                _ => bail!("no support for interface member type {:?} yet", member),
            }
        }
        Ok(decorator)
    }
}

// Represents an instance method for an object type.
//
// The FFI will represent this as a function whose first/self argument is a
// `FFIType::RustArcPtr` to the instance.
#[derive(Debug, Clone)]
pub struct DecoratorMethod {
    pub(super) name: String,
    pub(super) object_name: String,
    pub(super) return_type: Option<Type>,
    pub(super) attributes: MethodAttributes,
}

impl DecoratorMethod {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn return_type(&self) -> Option<&Type> {
        self.return_type.as_ref()
    }

    pub fn throws(&self) -> Option<&str> {
        self.attributes.get_throws_err()
    }

    pub fn throws_type(&self) -> Option<Type> {
        self.attributes
            .get_throws_err()
            .map(|name| Type::Error(name.to_owned()))
    }
}

impl Hash for DecoratorMethod {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.return_type.hash(state);
        self.attributes.hash(state);
    }
}

impl APIConverter<DecoratorMethod> for weedle::interface::OperationInterfaceMember<'_> {
    fn convert(&self, ci: &mut ComponentInterface) -> Result<DecoratorMethod> {
        if self.special.is_some() {
            bail!("special operations not supported");
        }
        if self.modifier.is_some() {
            bail!("method modifiers are not supported")
        }
        if !self.args.body.list.is_empty() {
            bail!("custom method arguments are not supported")
        }
        let return_type = ci.resolve_return_type_expression(&self.return_type)?;
        Ok(DecoratorMethod {
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
            return_type,
            attributes: MethodAttributes::try_from(self.attributes.as_ref())?,
        })
    }
}

#[cfg(test)]
mod test {
    use crate::interface::Type;

    use super::*;

    use super::super::object::{Method, Object};

    #[test]
    fn test_decorator_attribute_makes_a_decorator_object() {
        const UDL: &str = r#"
            namespace test{};
            [Decorator]
            interface Testing {
                sequence<u32> code_points_of_name();
            };
        "#;
        let ci = ComponentInterface::from_webidl(UDL).unwrap();
        assert_eq!(ci.iter_decorator_definitions().len(), 1);
        ci.get_decorator_definition("Testing").unwrap();
    }

    #[test]
    fn test_the_name_new_is_reserved_for_constructors() {
        const UDL: &str = r#"
            namespace test{};
            [Decorator]
            interface Testing {
                void new();
            };
        "#;
        let err = ComponentInterface::from_webidl(UDL).unwrap_err();
        assert_eq!(
            err.to_string(),
            "the method name \"new\" is reserved for the default constructor"
        );
    }

    #[test]
    fn test_methods_have_zero_args() {
        const UDL: &str = r#"
            namespace test{};
            [Decorator]
            interface Testing {
                void method(u32 arg);
            };
        "#;
        let err = ComponentInterface::from_webidl(UDL).unwrap_err();
        assert_eq!(err.to_string(), "custom method arguments are not supported");
    }

    #[test]
    fn test_decorator_methods_can_throw() {
        const UDL: &str = r#"
            namespace test{};
            [Decorator]
            interface Testing {
                [Throws=Error]
                void method();
            };

            [Error]
            enum Error {
                "LOLWUT"
            };
        "#;
        let ci = ComponentInterface::from_webidl(UDL).unwrap();
        let dobj = ci.get_decorator_definition("Testing").unwrap();
        let m = dobj.find_method("method").unwrap();

        assert_eq!(m.throws_type(), Some(Type::Error("Error".into())));
    }

    #[test]
    fn test_decorator_methods_can_override_return_types_and_throw_types() {
        const UDL: &str = r#"
            namespace test{};
            [Decorator]
            interface TheDecorator {
                [Throws=Error]
                void it_throws();

                Any? it_swallows();

                i32 it_counts();

                Any it_passes_through();
            };

            [Decorator=TheDecorator]
            interface Testing {
                [Throws=Error, CallsWith=it_swallows]
                void thrower();

                [CallsWith=it_throws]
                void silent();

                [CallsWith=it_swallows]
                i32 silent_with_return();

                [CallsWith=it_counts]
                void counted();

                [CallsWith=it_passes_through]
                sequence<i32?> exotic();
            };

            [Error]
            enum Error {
                "LOLWUT"
            };
        "#;
        let ci = ComponentInterface::from_webidl(UDL).unwrap();
        let dobj = ci.get_decorator_definition("TheDecorator");
        let obj = ci.get_object_definition("Testing").unwrap();

        fn find_method<'a>(nm: &str, obj: &'a Object) -> &'a Method {
            obj.methods.iter().find(|m| m.name() == nm).unwrap()
        }

        let m = find_method("thrower", obj);
        // thrower decorators through it_swallows, which returns void and throws nothing
        assert_eq!(m.decorated_return_type(&dobj), None);
        assert_eq!(m.decorated_throws_type(&dobj), None);

        let m = find_method("silent", obj);
        // silent decorators through it_throws, which returns void and throws nothing
        assert_eq!(m.decorated_return_type(&dobj), None);
        assert_eq!(
            m.decorated_throws_type(&dobj),
            Some(Type::Error("Error".into()))
        );

        let m = find_method("silent_with_return", obj);
        // silent decorators through it_throws, which returns void and throws nothing
        assert_eq!(
            m.decorated_return_type(&dobj),
            Some(Type::Optional(Box::new(Type::Int32)))
        );
        assert_eq!(m.decorated_throws_type(&dobj), None);

        let m = find_method("counted", obj);
        // counted decorators through it_counts, which returns i32 and throws nothing
        assert_eq!(m.decorated_return_type(&dobj), Some(Type::Int32));
        assert_eq!(m.decorated_throws_type(&dobj), None);

        let m = find_method("exotic", obj);
        // exotic decorators through it_passes_through, which returns Sequence<Option<i32>> and throws nothing
        assert_eq!(
            m.decorated_return_type(&dobj),
            Some(Type::Sequence(Box::new(Type::Optional(Box::new(
                Type::Int32
            )))))
        );
        assert_eq!(m.decorated_throws_type(&dobj), None);
    }
}
