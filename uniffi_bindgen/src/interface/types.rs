/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! # Basic typesystem for defining a component interface.
//!
//! Our typesystem operates at two distinct levels: "API-level types" and "FFI-level types".
//!
//! The [Type] enum represents high-level types that would appear in the public API of
//! a component, such as enums and records as well as primitives like ints and strings.
//! The rust code that implements a component, and the foreign language bindings that consume it,
//! will both typically deal with such types as their core concern.
//!
//! The [FFIType] enum represents low-level types that are used internally by our generated
//! code for passing data back and forth across the C-style FFI between rust and the foreign
//! language. These cover a much more restricted set of possible types. Consumers of a
//! uniffi component should not need to care about FFI-level types at all.
//!
//! As a developer working on uniffi itself, you're likely to spend a fair bit of time thinking
//! about how the API-level types map into FFI-level types and back again.
//!
//! The set of all [Type]s used in a component interface is represented by a [TypeUniverse],
//! which can be used by the bindings generator code to determine what type-related helper
//! functions to emit for a given component.

use anyhow::bail;
use anyhow::Result;
use std::convert::TryFrom;
use std::{collections::hash_map::Entry, collections::HashMap, collections::HashSet};

use super::Attributes;

/// Represents the restricted set of low-level types that can be used to construct
/// the C-style FFI layer between a rust component and its foreign language bindings.
///
/// For the types that involve memory allocation, we make a distinction between
/// "owned" types (the recipient must free it, or pass it to someone else) and
/// "borrowed" types (the sender must keep it alive for the duration of the call).
#[derive(Debug, Clone, Hash)]
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
    /// A `char*` pointer belonging to a rust-owned CString.
    /// If you've got one of these, you must call the appropriate rust function to free it.
    /// This is currently only used for error messages, and may go away in future.
    RustCString,
    /// A byte buffer allocated by rust, and owned by whoever currently holds it.
    /// If you've got one of these, you must either call the appropriate rust function to free it
    /// or pass it to someone that will.
    RustBuffer,
    /// A borrowed reference to some raw bytes owned by foreign language code.
    /// The provider of this reference must keep it alive for the duration of the receiving call.
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
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
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

impl Type {
    pub fn to_ffi(&self) -> FFIType {
        FFIType::from(self)
    }
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
            // Booleans lower into an Int8, to work around a bug in JNA.
            Type::Boolean => FFIType::Int8,
            // Strings are always owned rust values.
            // We might add a separate type for borrowed strings in future.
            Type::String => FFIType::RustBuffer,
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

impl Type {
    /// Get the canonical, unique-within-this-component name for a type.
    ///
    /// When generating helper code for foreign language bindings, it's sometimes useful to be
    /// able to name a particular type in order to e.g. call a helper function that is specific
    /// to that type. We support this by defining a naming convention where each type gets a
    /// unique canonical name, constructed recursively from the names of its component types (if any).
    pub fn canonical_name(&self) -> String {
        match self {
            // Builtin primitive types, with plain old names.
            Type::Int8 => "i8".into(),
            Type::UInt8 => "u8".into(),
            Type::Int16 => "i16".into(),
            Type::UInt16 => "u16".into(),
            Type::Int32 => "i32".into(),
            Type::UInt32 => "u32".into(),
            Type::Int64 => "i64".into(),
            Type::UInt64 => "u64".into(),
            Type::Float32 => "f32".into(),
            Type::Float64 => "f64".into(),
            Type::String => "string".into(),
            Type::Boolean => "bool".into(),
            // API defined types.
            // Note that these all get unique names, and the parser ensures that the names do not
            // conflict with a builtin type. We add a prefix to the name to guard against pathological
            // cases like a record named `SequenceRecord` interfering with `sequence<Record>`
            Type::Object(nm) => format!("Object{}", nm),
            Type::Error(nm) => format!("Error{}", nm),
            Type::Enum(nm) => format!("Enum{}", nm),
            Type::Record(nm) => format!("Record{}", nm),
            // Recursive types.
            // These add a prefix to the name of the underlying type.
            // The component API definition cannot give names to recursive types, so as long as the
            // prefixes we add here are all unique amongst themselves, then we have no chance of
            // acccidentally generating name collisions.
            Type::Optional(t) => format!("Optional{}", t.canonical_name()),
            Type::Sequence(t) => format!("Sequence{}", t.canonical_name()),
            Type::Map(t) => format!("Map{}", t.canonical_name()),
        }
    }
}

/// The set of all possible types used in a particular component interface.
///
/// Every component API uses a finite number of types, including primitive types, API-defined
/// types like records and enums, and recursive types such as sequences of the above. Our
/// component API doesn't support fancy generics so this is a finitely-enumerable set, which
/// is useful to be able to operate on explicitly.
///
/// You could imagine this struct doing some clever interning of names and so-on in future,
/// to reduce the overhead of passing around [Type] instances. For now we just do a whole
/// lot of cloning.
#[derive(Debug, Default)]
pub(crate) struct TypeUniverse {
    // Named type definitions (including aliases).
    type_definitions: HashMap<String, Type>,
    // All the types in the universe, by canonical type name.
    all_known_types: HashSet<Type>,
}

impl TypeUniverse {
    /// Add the definitions of all named [Type]s from a given WebIDL definition.
    ///
    /// This will fail if you try to add a name for which an existing type definition exists.
    pub(crate) fn add_type_definitions_from<T: TypeFinder>(&mut self, defn: T) -> Result<()> {
        defn.add_type_definitions_to(self)
    }

    /// Add the definition of a named [Type].
    ///
    /// This will fail if you try to add a name for which an existing type definition exists.
    pub fn add_type_definition(&mut self, name: &str, type_: Type) -> Result<()> {
        if resolve_builtin_type(name).is_some() {
            bail!(
                "please don't shadow builtin types ({}, {})",
                name,
                type_.canonical_name()
            );
        }
        let type_ = self.add_known_type(type_)?;
        match self.type_definitions.entry(name.to_string()) {
            Entry::Occupied(_) => bail!("Conflicting type definition for {}", name),
            Entry::Vacant(e) => {
                e.insert(type_);
                Ok(())
            }
        }
    }

    /// Get the [Type] corresponding to a given name, if any.
    fn get_type_definition(&self, name: &str) -> Option<Type> {
        self.type_definitions.get(name).cloned()
    }

    /// Get the [Type] corresponding to a given WebIDL type node.
    ///
    /// If the node is a structural type (e.g. a sequence) then this will also add
    /// it to the set of all types seen in the component interface.
    pub(crate) fn resolve_type_expression<T: TypeResolver>(&mut self, expr: T) -> Result<Type> {
        expr.resolve_type_expression(self)
    }

    /// Add a [Type] to the set of all types seen in the component interface.
    ///
    /// This helpfully returns a `Result<Type>` so it can be chained in with other
    /// methods during the type resolution process.
    pub fn add_known_type(&mut self, type_: Type) -> Result<Type> {
        // Types are more likely to already be known than not, so avoid unnecessary cloning.
        if !self.all_known_types.contains(&type_) {
            self.all_known_types.insert(type_.clone());
        }
        Ok(type_)
    }

    /// Iterator over all the known types in this universe.
    pub fn iter_known_types(&self) -> impl Iterator<Item = Type> + '_ {
        self.all_known_types.iter().cloned()
    }
}

/// Trait to help with an early "type discovery" phase when processing the UDL.
///
/// Ths trait does structural matching against weedle AST nodes from a parsed
/// UDL file, looking for all the newly-defined types in the file and accumulating
/// them in the given `TypeUniverse`. Doing this in a preliminary pass means that
/// we know how to resolve names to types when building up the full interface
/// definition.
pub(crate) trait TypeFinder {
    fn add_type_definitions_to(&self, types: &mut TypeUniverse) -> Result<()>;
}

impl<T: TypeFinder> TypeFinder for &[T] {
    fn add_type_definitions_to(&self, types: &mut TypeUniverse) -> Result<()> {
        for item in self.iter() {
            (&item).add_type_definitions_to(types)?;
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
            _ => Ok(()),
        }
    }
}

impl TypeFinder for weedle::InterfaceDefinition<'_> {
    fn add_type_definitions_to(&self, types: &mut TypeUniverse) -> Result<()> {
        let name = self.identifier.0.to_string();
        types.add_type_definition(self.identifier.0, Type::Object(name))
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
        if let Some(attrs) = &self.attributes {
            let attrs = Attributes::try_from(attrs)?;
            if attrs.contains_error_attr() {
                return types.add_type_definition(self.identifier.0, Type::Error(name));
            }
        }
        types.add_type_definition(self.identifier.0, Type::Enum(name))
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

/// Trait to help resolving an UDL type node to a [Type].
///
/// Ths trait does structural matching against type-related weedle AST nodes from
/// a parsed UDL file, turning them into a corresponding [Type] struct. It uses the
/// known type definitions in a [TypeUniverse] to resolve names, and so it assumes
/// that we've already done a [TypeFinder] pass.
///
/// (And to be honest, a big part of its job is to error out if we encounter any of the
/// many, many WebIDL type definitions that are not supported by uniffi.)
pub(crate) trait TypeResolver {
    fn resolve_type_expression(&self, types: &mut TypeUniverse) -> Result<Type>;
}

impl TypeResolver for &weedle::types::Type<'_> {
    fn resolve_type_expression(&self, types: &mut TypeUniverse) -> Result<Type> {
        (*self).resolve_type_expression(types)
    }
}

impl TypeResolver for weedle::types::Type<'_> {
    fn resolve_type_expression(&self, types: &mut TypeUniverse) -> Result<Type> {
        match self {
            weedle::types::Type::Single(t) => match t {
                weedle::types::SingleType::Any(_) => bail!("no support for `any` types"),
                weedle::types::SingleType::NonAny(t) => t.resolve_type_expression(types),
            },
            weedle::types::Type::Union(_) => bail!("no support for union types yet"),
        }
    }
}

impl TypeResolver for weedle::types::NonAnyType<'_> {
    fn resolve_type_expression(&self, types: &mut TypeUniverse) -> Result<Type> {
        match self {
            weedle::types::NonAnyType::Boolean(t) => t.resolve_type_expression(types),
            weedle::types::NonAnyType::Identifier(t) => t.resolve_type_expression(types),
            weedle::types::NonAnyType::Integer(t) => t.resolve_type_expression(types),
            weedle::types::NonAnyType::FloatingPoint(t) => t.resolve_type_expression(types),
            weedle::types::NonAnyType::Sequence(t) => t.resolve_type_expression(types),
            weedle::types::NonAnyType::RecordType(t) => t.resolve_type_expression(types),
            _ => bail!("no support for type {:?}", self),
        }
    }
}

impl TypeResolver for &weedle::types::AttributedNonAnyType<'_> {
    fn resolve_type_expression(&self, types: &mut TypeUniverse) -> Result<Type> {
        if self.attributes.is_some() {
            bail!("type attributes are not supported yet");
        }
        (&self.type_).resolve_type_expression(types)
    }
}

impl TypeResolver for &weedle::types::AttributedType<'_> {
    fn resolve_type_expression(&self, types: &mut TypeUniverse) -> Result<Type> {
        if self.attributes.is_some() {
            bail!("type attributes are not supported yet");
        }
        (&self.type_).resolve_type_expression(types)
    }
}

impl<T: TypeResolver> TypeResolver for weedle::types::MayBeNull<T> {
    fn resolve_type_expression(&self, types: &mut TypeUniverse) -> Result<Type> {
        let type_ = self.type_.resolve_type_expression(types)?;
        match self.q_mark {
            None => Ok(type_),
            Some(_) => types.add_known_type(Type::Optional(Box::new(type_))),
        }
    }
}

impl TypeResolver for weedle::types::IntegerType {
    fn resolve_type_expression(&self, _types: &mut TypeUniverse) -> Result<Type> {
        bail!(
            "WebIDL integer types not implemented ({:?}); consider using u8, u16, u32 or u64",
            self
        )
    }
}

impl TypeResolver for weedle::types::FloatingPointType {
    fn resolve_type_expression(&self, types: &mut TypeUniverse) -> Result<Type> {
        match self {
            weedle::types::FloatingPointType::Float(t) => t.resolve_type_expression(types),
            weedle::types::FloatingPointType::Double(t) => t.resolve_type_expression(types),
        }
    }
}

impl TypeResolver for weedle::types::SequenceType<'_> {
    fn resolve_type_expression(&self, types: &mut TypeUniverse) -> Result<Type> {
        let t = self.generics.body.as_ref().resolve_type_expression(types)?;
        types.add_known_type(Type::Sequence(Box::new(t)))
    }
}

impl TypeResolver for weedle::types::RecordType<'_> {
    fn resolve_type_expression(&self, types: &mut TypeUniverse) -> Result<Type> {
        let t = (&self.generics.body.2).resolve_type_expression(types)?;
        // Maps always have string keys, make sure the `String` type is known.
        types.add_known_type(Type::String)?;
        types.add_known_type(Type::Map(Box::new(t)))
    }
}

impl TypeResolver for weedle::common::Identifier<'_> {
    fn resolve_type_expression(&self, types: &mut TypeUniverse) -> Result<Type> {
        match resolve_builtin_type(self.0) {
            Some(type_) => types.add_known_type(type_),
            None => match types.get_type_definition(self.0) {
                Some(type_) => types.add_known_type(type_),
                None => bail!("unknown type reference: {}", self.0),
            },
        }
    }
}

impl TypeResolver for weedle::term::Boolean {
    fn resolve_type_expression(&self, types: &mut TypeUniverse) -> Result<Type> {
        types.add_known_type(Type::Boolean)
    }
}

impl TypeResolver for weedle::types::FloatType {
    fn resolve_type_expression(&self, types: &mut TypeUniverse) -> Result<Type> {
        if self.unrestricted.is_some() {
            bail!("we don't support `unrestricted float`");
        }
        types.add_known_type(Type::Float32)
    }
}

impl TypeResolver for weedle::types::DoubleType {
    fn resolve_type_expression(&self, types: &mut TypeUniverse) -> Result<Type> {
        if self.unrestricted.is_some() {
            bail!("we don't support `unrestricted double`");
        }
        types.add_known_type(Type::Float64)
    }
}

/// Resolve built-in API types by name.
///
/// Given an identifier from the UDL, this will return `Some(Type)` if it names one of the
/// built-in primitive types or `None` if it names something else.
fn resolve_builtin_type(name: &str) -> Option<Type> {
    match name {
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
