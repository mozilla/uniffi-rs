/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! # Basic typesystem for defining a component interface.
//!
//! Our typesystem operates at two distinct levels: "API-level types" and "FFI-level types".
//!
//! The [`Type`](Type) enum represents high-level types that would appear in the public API
//! of a component, such as enums and records as well as primitives like ints and strings.
//! The rust code that implements a component, and the foreign language bindings that consume it,
//! will both typically deal with such types as their core concern.
//!
//! The [`FFIType`](FFIType) enum represents low-level types that are used internally by our
//! generated code for passing data back and forth across the C-style FFI between rust and the
//! foreign language. These cover a much more restricted set of possible types. Consumers of a
//! uniffi component should not need to care about FFI-level types at all.
//!
//! As a developer working on uniffi itself, you're likely to spend a fair bit of time thinking
//! about how the API-level types map into FFI-level types and back again.

use anyhow::bail;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

use super::{Attributes, ComponentInterface};

/// Represents the restricted set of low-level types that can be used to construct
/// the C-style FFI layer between a rust component and its foreign language bindings.
///
/// For the types that involve memory allocation, we make a distinction between
/// "owned" types (the recipient must free it, or pass it to someone else) and
/// "borrowed" types (the sender must keep it alive for the duration of the call).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FFIType {
    // N.B. there are no booleans at this layer, since they cause problems for JNA.
    UInt8,
    Int8,
    UInt16,
    Int16,
    UInt32,
    Int32,
    UInt64,
    Int64,
    Float32,
    Float64,
    /// A byte buffer allocated by rust, and owned by whoever currently holds it.
    /// If you've got one of these, you must either call the appropriate rust function to free it
    /// or pass it to someone that will.
    RustBuffer,
    /// A UTF-8 string buffer allocated by rust, and owned by whoever currently holds it.
    /// If you've got one of these, you must either call the appropriate rust function to free it
    /// or pass it to someone that will.
    RustString,
    /// A borrowed reference to a UTF-8 string buffer owned by foreign language code.
    /// A borrowed reference to some raw bytes owned by foreign language code.
    /// The provider of this reference must keep it alive for the duration of the receiving call.
    ForeignStringRef,
    ForeignBytes,
    /// An error struct, containing a numberic error code and char* pointer to error string.
    /// The string is owned by rust and allocated on the rust heap, and must be freed by
    /// passing it to the appropriate `string_free` FFI function.
    RustError,
    // TODO: you can imagine a richer structural typesystem here, e.g. `Ref<String>` or something.
    // We don't need that yet and it's possible we never will, so it isn't here for now.
}

/// Represents all the different high-level types that can be used in a component interface.
/// At this level we identify user-defined types by name, without knowing any details
/// of their internal structure apart from what type of thing they are (record, enum, etc).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Type {
    // Primitive types.
    UInt8,
    Int8,
    UInt16,
    Int16,
    UInt32,
    Int32,
    UInt64,
    Int64,
    Float32,
    Float64,
    Boolean,
    String,
    // Types defined in the component API, each of which has a string name.
    Object(String),
    Record(String),
    Enum(String),
    Error(String),
    // Structurally recursive types.
    Optional(Box<Type>),
    Sequence(Box<Type>),
    Map(/* String, */ Box<Type>),
}

/// When passing data across the FFI, each `Type` value will be lowered into a corresponding
/// `FFIType` value. This conversion tells you which one.
impl From<&Type> for FFIType {
    fn from(v: &Type) -> FFIType {
        match v {
            // Types that are the same map to themselves, naturally.
            Type::UInt8 => FFIType::UInt8,
            Type::Int8 => FFIType::Int8,
            Type::UInt16 => FFIType::UInt16,
            Type::Int16 => FFIType::Int16,
            Type::UInt32 => FFIType::UInt32,
            Type::Int32 => FFIType::Int32,
            Type::UInt64 => FFIType::UInt64,
            Type::Int64 => FFIType::Int64,
            Type::Float32 => FFIType::Float32,
            Type::Float64 => FFIType::Float64,
            // Booleans lower into a byte, to work around a bug in JNA.
            Type::Boolean => FFIType::UInt8,
            // Strings are always owned rust values.
            // We might add a separate type for borrowed strings in future.
            Type::String => FFIType::RustString,
            // Objects are passed as opaque integer handles.
            Type::Object(_) => FFIType::UInt64,
            // Enums are passed as integers.
            Type::Enum(_) => FFIType::UInt32,
            // Errors have their own special type.
            Type::Error(_) => FFIType::RustError,
            // Other types are serialized into a bytebuffer and deserialized on the other side.
            Type::Record(_) | Type::Optional(_) | Type::Sequence(_) | Type::Map(_) => {
                FFIType::RustBuffer
            }
        }
    }
}

/// Trait to help with an early "type discovery" phase when processing the IDL.
///
/// Ths trait does structural matching against weedle AST nodes from a parsed
/// IDL file, looking for all the named types defined in the file and accumulating
/// them in the `ComponentInterface`. Doing this in a preliminary pass means that
/// we know how to resolve names to types when building up the full interface
/// definition.
pub(crate) trait TypeFinder {
    fn find_type_definitions(&self, ci: &mut ComponentInterface) -> Result<()>;
}

impl<T: TypeFinder> TypeFinder for Vec<T> {
    fn find_type_definitions(&self, ci: &mut ComponentInterface) -> Result<()> {
        for item in self.iter() {
            (&item).find_type_definitions(ci)?;
        }
        Ok(())
    }
}

impl TypeFinder for weedle::Definition<'_> {
    fn find_type_definitions(&self, ci: &mut ComponentInterface) -> Result<()> {
        match self {
            weedle::Definition::Interface(d) => d.find_type_definitions(ci),
            weedle::Definition::Dictionary(d) => d.find_type_definitions(ci),
            weedle::Definition::Enum(d) => d.find_type_definitions(ci),
            weedle::Definition::Typedef(d) => d.find_type_definitions(ci),
            _ => Ok(()),
        }
    }
}

impl TypeFinder for weedle::InterfaceDefinition<'_> {
    fn find_type_definitions(&self, ci: &mut ComponentInterface) -> Result<()> {
        let name = self.identifier.0.to_string();
        if resolve_builtin_type(&self.identifier).is_some() {
            bail!("please don't shadow builtin types (interface {})", name);
        }
        ci.add_type_definition(self.identifier.0, Type::Object(name))
    }
}

impl TypeFinder for weedle::DictionaryDefinition<'_> {
    fn find_type_definitions(&self, ci: &mut ComponentInterface) -> Result<()> {
        let name = self.identifier.0.to_string();
        if resolve_builtin_type(&self.identifier).is_some() {
            bail!("please don't shadow builtin types (dictionary {})", name);
        }
        ci.add_type_definition(self.identifier.0, Type::Record(name))
    }
}

impl TypeFinder for weedle::EnumDefinition<'_> {
    fn find_type_definitions(&self, ci: &mut ComponentInterface) -> Result<()> {
        if let Some(attrs) = &self.attributes {
            let attrs = Attributes::try_from(attrs)?;
            if attrs.contains_error_attr() {
                let name = self.identifier.0.to_string();
                return ci.add_type_definition(self.identifier.0, Type::Error(name));
            }
        }
        let name = self.identifier.0.to_string();
        if resolve_builtin_type(&self.identifier).is_some() {
            bail!("please don't shadow builtin types (enum {})", name);
        }
        ci.add_type_definition(self.identifier.0, Type::Enum(name))
    }
}

impl TypeFinder for weedle::TypedefDefinition<'_> {
    fn find_type_definitions(&self, ci: &mut ComponentInterface) -> Result<()> {
        if self.attributes.is_some() {
            bail!("no typedef attributes are currently supported");
        }
        match resolve_builtin_type(&self.identifier) {
            Some(_) => bail!(
                "please don't try to redefine builtin types, kthxbye ({:?})",
                self
            ),
            None => {
                // For now, we assume that the typedef must refer to an already-defined type, which means
                // we can look it up in the ComponentInterface. This should suffice for our needs for
                // a good long while before we consider implementing a more complex delayed resolution strategy.
                ci.add_type_definition(self.identifier.0, self.type_.resolve_type_definition(ci)?)
            }
        }
    }
}

/// Trait to help resolving an IDL type node to a `HighLevelType`.
///
/// Ths trait does structural matching against type-related weedle AST nodes from
/// a parsed IDL file, turning them into a corresponding `HighLevelType`. It uses the
/// mapping from names to types previous built by the `TypeFinder` trait in order to
/// resolve types by name.
///
/// (And to be honest, a big part of its job is to error out if we encounter any of the
/// many, many WebIDL type definitions that are not supported by uniffi.)
pub(crate) trait TypeResolver {
    fn resolve_type_definition(&self, ci: &ComponentInterface) -> Result<Type>;
}

impl TypeResolver for &weedle::types::Type<'_> {
    fn resolve_type_definition(&self, ci: &ComponentInterface) -> Result<Type> {
        (*self).resolve_type_definition(ci)
    }
}

impl TypeResolver for weedle::types::Type<'_> {
    fn resolve_type_definition(&self, ci: &ComponentInterface) -> Result<Type> {
        match self {
            weedle::types::Type::Single(t) => match t {
                weedle::types::SingleType::Any(_) => bail!("no support for `any` types"),
                weedle::types::SingleType::NonAny(t) => t.resolve_type_definition(ci),
            },
            weedle::types::Type::Union(_) => bail!("no support for union types yet"),
        }
    }
}

impl TypeResolver for weedle::types::NonAnyType<'_> {
    fn resolve_type_definition(&self, ci: &ComponentInterface) -> Result<Type> {
        match self {
            weedle::types::NonAnyType::Boolean(t) => t.resolve_type_definition(ci),
            weedle::types::NonAnyType::Identifier(t) => t.resolve_type_definition(ci),
            weedle::types::NonAnyType::Integer(t) => t.resolve_type_definition(ci),
            weedle::types::NonAnyType::FloatingPoint(t) => t.resolve_type_definition(ci),
            weedle::types::NonAnyType::Sequence(t) => t.resolve_type_definition(ci),
            weedle::types::NonAnyType::RecordType(t) => t.resolve_type_definition(ci),
            _ => bail!("no support for type {:?}", self),
        }
    }
}

impl TypeResolver for weedle::types::AttributedNonAnyType<'_> {
    fn resolve_type_definition(&self, ci: &ComponentInterface) -> Result<Type> {
        if self.attributes.is_some() {
            bail!("type attributes are not supported yet");
        }
        (&self.type_).resolve_type_definition(ci)
    }
}

impl TypeResolver for weedle::types::AttributedType<'_> {
    fn resolve_type_definition(&self, ci: &ComponentInterface) -> Result<Type> {
        if self.attributes.is_some() {
            bail!("type attributes are not supported yet");
        }
        (&self.type_).resolve_type_definition(ci)
    }
}

impl<T: TypeResolver> TypeResolver for weedle::types::MayBeNull<T> {
    fn resolve_type_definition(&self, ci: &ComponentInterface) -> Result<Type> {
        let type_ = self.type_.resolve_type_definition(ci)?;
        Ok(match self.q_mark {
            None => type_,
            Some(_) => Type::Optional(Box::new(type_)),
        })
    }
}

impl TypeResolver for weedle::types::IntegerType {
    fn resolve_type_definition(&self, _ci: &ComponentInterface) -> Result<Type> {
        bail!(
            "integer types not implemented ({:?}); consider using u8, u16, u32 or u64",
            self
        )
    }
}

impl TypeResolver for weedle::types::FloatingPointType {
    fn resolve_type_definition(&self, ci: &ComponentInterface) -> Result<Type> {
        match self {
            weedle::types::FloatingPointType::Float(t) => t.resolve_type_definition(ci),
            weedle::types::FloatingPointType::Double(t) => t.resolve_type_definition(ci),
        }
    }
}

impl TypeResolver for weedle::types::SequenceType<'_> {
    fn resolve_type_definition(&self, ci: &ComponentInterface) -> Result<Type> {
        Ok(Type::Sequence(Box::new(
            self.generics.body.as_ref().resolve_type_definition(ci)?,
        )))
    }
}

impl TypeResolver for weedle::types::RecordType<'_> {
    fn resolve_type_definition(&self, ci: &ComponentInterface) -> Result<Type> {
        Ok(Type::Map(Box::new(
            (&self.generics.body.2).resolve_type_definition(ci)?,
        )))
    }
}

impl TypeResolver for weedle::common::Identifier<'_> {
    fn resolve_type_definition(&self, ci: &ComponentInterface) -> Result<Type> {
        match resolve_builtin_type(self) {
            Some(type_) => Ok(type_),
            None => match ci.get_type_definition(self.0) {
                Some(type_) => Ok(type_),
                None => bail!("unknown type reference: {}", self.0),
            },
        }
    }
}

impl TypeResolver for weedle::term::Boolean {
    fn resolve_type_definition(&self, _ci: &ComponentInterface) -> Result<Type> {
        Ok(Type::Boolean)
    }
}

impl TypeResolver for weedle::types::FloatType {
    fn resolve_type_definition(&self, _ci: &ComponentInterface) -> Result<Type> {
        if self.unrestricted.is_some() {
            bail!("we don't support `unrestricted float`");
        }
        Ok(Type::Float32)
    }
}

impl TypeResolver for weedle::types::DoubleType {
    fn resolve_type_definition(&self, _ci: &ComponentInterface) -> Result<Type> {
        if self.unrestricted.is_some() {
            bail!("we don't support `unrestricted double`");
        }
        Ok(Type::Float64)
    }
}

/// Resolve built-in API types by name.
/// Given an identifier from the IDL, this will return `Some(Type)` if it names one of the
/// built-in primitive types or `None` if it names something else.
fn resolve_builtin_type(id: &weedle::common::Identifier<'_>) -> Option<Type> {
    match id.0 {
        "string" => Some(Type::String),
        "u8" => Some(Type::UInt8),
        "i8" => Some(Type::Int8),
        "u16" => Some(Type::UInt16),
        "i16" => Some(Type::Int16),
        "u32" => Some(Type::UInt32),
        "i32" => Some(Type::Int32),
        "u64" => Some(Type::UInt64),
        "i64" => Some(Type::Int64),
        "f32" => Some(Type::Float32),
        "f64" => Some(Type::Float64),
        _ => None,
    }
}
