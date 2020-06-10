/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//#![deny(missing_docs)]
#![allow(unknown_lints)]
#![warn(rust_2018_idioms)]

//! # Component Interface Definition and Types
//!
//! This crate provides an abstract representation of the interface provided by a "rust component",
//! in high-level terms suitable for translation into target consumer languages such as Kotlin
//! and swift. It also provides facilities for parsing a WebIDL interface definition file into such a
//! representation.
//!
//! The entrypoint to this crate is the `ComponentInterface` struct, which holds a complete definition
//! of the interface provided by a component. That's really the key concept of this crate so it's
//! worth repeating: a `ComponentInterface` completely defines the shape and semantics of an interface
//! between the rust-based implementation of a component and the foreign language consumers, including
//! details like:
//!
//!    * The names of all symbols in the compiled object file
//!    * The type and arity of all exported functions
//!    * The layout and conventions used for all arguments and return types
//!
//! If you have a dynamic library compiled from a rust component using this crate, and a foreign
//! language binding generated from the same `ComponentInterface` using the same version of this
//! crate, then there should be no opportunities for them to disagree on how the two sides of the
//! FFI boundary interact.
//!
//! Docs TODOS:
//!   * define "rust component" for someone who doesn't already know what it is
//!

use std::io::prelude::*;
use std::{
    env,
    collections::HashMap,
    convert::TryFrom, convert::TryInto,
    fs::File,
    path::{Path, PathBuf},
    str::FromStr,
};

use anyhow::anyhow;
use anyhow::bail;
use anyhow::Result;

// We make extensive use of `TryInto` for converting WebIDL types parsed by weedle into the
// simpler types exported by this create. This is a little helper to simply doing that conversion
// on a list of objects and either collecting the result, or propagating the first error.
macro_rules! try_into_collection {
    ($e:expr) => {
        ($e).iter().map(TryInto::try_into).collect::<Result<_,_>>()
    }
}

/// The main public interface for this module, representing the complete details of an FFI that will
/// be exposed by a rust component (or equivalently, that will be consumed by foreign language bindings).
///
/// We're still figuring out the right shape of the abstraction here; currently it just contains a
/// Vec of individual interface members, but it'll almost certainly need to grow metadata about the
/// interface as a whole.
///
/// TODOs:
///   * probably we can have the `ComponentInterface` own a bunch of interned strings, and have its
///     members just hold references to them. This could reduce a lot of `clone`ing in the code below.
#[derive(Debug)]
pub struct ComponentInterface {
    /// The individual members (objects, records, etc) that make the component interface.
    pub members: Vec<InterfaceMember>,

}

impl ComponentInterface {

    /// Parse a `ComponentInterface` from a string containing a WebIDL definition.
    pub fn from_webidl(idl: &str) -> Result<Self> {
        // There's some lifetime thing with the errors returned from weedle::parse
        // that life is too short to figure out; unwrap and move on.
        Ok(Self { members: try_into_collection!(weedle::parse(idl.trim()).unwrap())? })
    }
}

impl FromStr for ComponentInterface {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self> {
        ComponentInterface::from_webidl(s)
    }
}

/// A component interface consists of four different types of definition:
///
///    - Objects, opaque types with methods that can be instantiated and passed
///      around by reference.
///    - Records, types that hold data and are typically passed aroud by value.
///    - Namespaces, collections of standlone functions.
///    - Enums, simple lists of named options.
///
/// The list of top-level members will almost certainly grow, but this is what
/// we have for now.
#[derive(Debug)]
pub enum InterfaceMember {
    Object(ObjectType),
    Record(RecordType),
    Namespace(NamespaceType),
    Enum(EnumType),
}

impl InterfaceMember {
    fn name(&self) -> &str{
        match self {
            InterfaceMember::Object(t) => &t.name,
            InterfaceMember::Record(t) => &t.name,
            InterfaceMember::Namespace(t) => &t.name,
            InterfaceMember::Enum(t) => &t.name,
        }
    }
}

impl TryFrom<&weedle::Definition<'_>> for InterfaceMember {
    type Error = anyhow::Error;
    fn try_from(d: &weedle::Definition) -> Result<Self> {
        Ok(match d {
            weedle::Definition::Interface(d) => InterfaceMember::Object(d.try_into()?),
            weedle::Definition::Dictionary(d) => InterfaceMember::Record(d.try_into()?),
            weedle::Definition::Namespace(d) => InterfaceMember::Namespace(d.try_into()?),
            weedle::Definition::Enum(d) => InterfaceMember::Enum(d.try_into()?),
            _ => bail!("don't know how to deal with {:?}", d),
        })
    }
}

/// An "object" is an opaque type that can be instantiated and passed around by reference,
/// have methods called on it, and so on - basically your classic Object Oriented Programming
/// type of deal, except without elaborate inheritence hierarchies.
///
/// In WebIDL these correspond to the `interface` keyword.
///
/// At the FFI layer, objects are represented by an opaque integer handle and a set of functions
/// a common prefix. The object's constuctors are functions that return new objects by handle,
/// and its methods are functions that take a handle as first argument. The foreign language
/// binding code is expected to stitch these functions back together into an appropriate class
/// definition (or that language's equivalent thereof).
///
/// TODO:
///  - maybe "ClassType" would be a better name than "ObjectType" here?
#[derive(Debug)]
pub struct ObjectType {
    pub name: String,
    pub members: Vec<ObjectTypeMember>,
}

impl TryFrom<&weedle::InterfaceDefinition<'_>> for ObjectType {
    type Error = anyhow::Error;
    fn try_from(d: &weedle::InterfaceDefinition) -> Result<Self> {
        if d.attributes.is_some() {
            bail!("no interface attributes are supported yet");
        }
        if d.inheritance.is_some() {
            bail!("interface inheritence is not supported");
        }
        Ok(ObjectType {
            name: d.identifier.0.to_string(),
            members: try_into_collection!(d.members.body)?
        })
    }
}

// Represets the different types of members that can be part of an individual object's interface:
//  - constructors
//  - instance methods
//
// We'll probably grow more here over time, e.g. maybe static methods should be separate?
#[derive(Debug)]
pub enum ObjectTypeMember {
    Constructor(ObjectTypeConstructor),
    Method(ObjectTypeMethod)
}

impl TryFrom<&weedle::interface::InterfaceMember<'_>> for ObjectTypeMember {
    type Error = anyhow::Error;
    fn try_from(m: &weedle::interface::InterfaceMember) -> Result<Self> {
        Ok(match m {
            weedle::interface::InterfaceMember::Constructor(t) => ObjectTypeMember::Constructor(t.try_into()?),
            weedle::interface::InterfaceMember::Operation(t) => ObjectTypeMember::Method(t.try_into()?),
            _ => bail!("no support for interface member type {:?} yet", m),
        })
    }
}

// Represents a constructor for an object type.
//
// In the FFI, this will be a function that returns a handle for an instance
// of the corresponding object type.
#[derive(Debug)]
pub struct ObjectTypeConstructor {
    pub name: String,
    pub argument_types: Vec<ObjectTypeArgument>,
}


impl ObjectTypeConstructor {
    pub fn ffi_name(&self) -> String {
        self.name.clone() // XXX TODO: calculate prefix from the containing Object declaration, somehow.
    }
}

impl TryFrom<&weedle::interface::ConstructorInterfaceMember<'_>> for ObjectTypeConstructor {
    type Error = anyhow::Error;
    fn try_from(m: &weedle::interface::ConstructorInterfaceMember) -> Result<Self> {
        if m.attributes.is_some() {
            bail!("no interface member attributes supported yet");
        }
        Ok(ObjectTypeConstructor {
            name: String::from("new"), // TODO: get the name from an attribute maybe?
            argument_types: try_into_collection!(m.args.body.list)?
        })
    }
}

// Represents an instance method for an object type.
//
// The in FFI, this will be a function whose first argument is a handle for an
// instance of the corresponding object type.
#[derive(Debug)]
pub struct ObjectTypeMethod {
    pub name: String,
    pub return_type: Option<UnresolvedTypeReference>,
    pub argument_types: Vec<ObjectTypeArgument>,
}

impl ObjectTypeMethod {
    pub fn ffi_name(&self) -> String {
        let mut nm = String::from("fxa_"); // XXX TODO: calculate prefix from the containing Object declaration, somehow.
        nm.push_str(&self.name);
        nm
    }
}

impl TryFrom<&weedle::interface::OperationInterfaceMember<'_>> for ObjectTypeMethod {
    type Error = anyhow::Error;
    fn try_from(m: &weedle::interface::OperationInterfaceMember) -> Result<Self> {
        if m.attributes.is_some() {
            bail!("no interface member attributes supported yet");
        }
        if m.special.is_some() {
            bail!("special operations not supported");
        }
        if let Some(weedle::interface::StringifierOrStatic::Stringifier(_)) = m.modifier {
            bail!("stringifiers are not supported");
        }
        if let None = m.identifier {
            bail!("anonymous methods are not supported {:?}", m);
        }
        Ok(ObjectTypeMethod {
            name: m.identifier.unwrap().0.to_string(),
            return_type: match &m.return_type {
                weedle::types::ReturnType::Void(_) => None,
                weedle::types::ReturnType::Type(t) => Some(t.try_into()?)
            },
            argument_types: try_into_collection!(m.args.body.list)?
        })
    }
}

// Represents an argument to an object constructor or method call.
//
// Each argument has a name and a type, along with some optional
#[derive(Debug)]
pub struct ObjectTypeArgument {
    pub name: String,
    pub typ: UnresolvedTypeReference,
    pub optional: bool,
    pub default: Option<Literal>,
}

impl ObjectTypeArgument {
    pub fn ffi_name(&self) -> String {
        self.name.to_string()
    }
}

impl TryFrom<&weedle::argument::Argument<'_>> for ObjectTypeArgument {
    type Error = anyhow::Error;
    fn try_from(t: &weedle::argument::Argument) -> Result<Self> {
        Ok(match t {
            weedle::argument::Argument::Single(t) => t.try_into()?,
            weedle::argument::Argument::Variadic(_) => bail!("variadic arguments not supported"),
        })
    }
}

impl TryFrom<&weedle::argument::SingleArgument<'_>> for ObjectTypeArgument {
    type Error = anyhow::Error;
    fn try_from(a: &weedle::argument::SingleArgument) -> Result<Self> {
        if a.attributes.is_some() {
            bail!("no argument attributes supported yet");
        }
        Ok(ObjectTypeArgument {
            name: a.identifier.0.to_string(),
            typ: (&a.type_).try_into()?,
            optional: a.optional.is_some(),
            default: a.default.map(|v| v.value.try_into().unwrap())
        })
    }
}


// Represents a "data class" style object, for passing around complex values.
// In the FFI these are represented as a ByteBuffer, which one side explicitly
// serializes the data into and the other serializes it out of. So I guess they're
// kind of like "pass by clone" values.
#[derive(Debug, Default)]
pub struct RecordType {
    pub name: String,
    pub fields: Vec<RecordTypeField>,
}

impl TryFrom<&weedle::DictionaryDefinition<'_>> for RecordType {
    type Error = anyhow::Error;
    fn try_from(d: &weedle::DictionaryDefinition) -> Result<Self> {
        if d.attributes.is_some() {
            bail!("no dictionary attributes are supported yet");
        }
        if d.inheritance.is_some() {
            bail!("dictionary inheritence is not support");
        }
        Ok(RecordType {
            name: d.identifier.0.to_string(),
            fields: try_into_collection!(d.members.body)?,
        })
    }
}

// Represents an individual field on a Record.
#[derive(Debug)]
pub struct RecordTypeField {
    pub name: String,
    pub typ: UnresolvedTypeReference,
    pub required: bool,
    pub default: Option<Literal>,
}

impl TryFrom<&weedle::dictionary::DictionaryMember<'_>> for RecordTypeField {
    type Error = anyhow::Error;
    fn try_from(d: &weedle::dictionary::DictionaryMember) -> Result<Self> {
        if d.attributes.is_some() {
            bail!("no dictionary member attributes are supported yet");
        }
        Ok(Self {
            name: d.identifier.0.to_string(),
            typ: (&d.type_).try_into()?,
            required: d.required.is_some(),
            default: match d.default {
                None => None,
                Some(v) => Some(v.value.try_into()?),
            },
        })
    }
}

/// A namespace is simply a collection of stand-alone functions. It looks similar to
/// an interface but cannot be instantiated.
#[derive(Debug)]
pub struct NamespaceType {
    pub name: String,
    pub members: Vec<NamespaceTypeMember>,
}

impl NamespaceType {
    pub fn struct_name(&self) -> String {
        self.name.to_string()
    }
}

impl TryFrom<&weedle::NamespaceDefinition<'_>> for NamespaceType {
    type Error = anyhow::Error;
    fn try_from(d: &weedle::NamespaceDefinition) -> Result<Self> {
        if d.attributes.is_some() {
            bail!("no interface attributes are supported yet");
        }
        Ok(NamespaceType {
            name: d.identifier.0.to_string(),
            members: try_into_collection!(d.members.body)?
        })
    }
}

// Represets the different types of members that can be part of a namespace.
//  - currently, only functions.
#[derive(Debug)]
pub enum NamespaceTypeMember {
    Function(NamespaceTypeFunction)
}

impl TryFrom<&weedle::namespace::NamespaceMember<'_>> for NamespaceTypeMember {
    type Error = anyhow::Error;
    fn try_from(m: &weedle::namespace::NamespaceMember) -> Result<Self> {
        Ok(match m {
            weedle::namespace::NamespaceMember::Operation(t) => NamespaceTypeMember::Function(t.try_into()?),
            _ => bail!("no support for namespace member type {:?} yet", m),
        })
    }
}


// Represents an individual function in a namespace.
//
// The in FFI, this will be a standalone function.
#[derive(Debug)]
pub struct NamespaceTypeFunction {
    pub name: String,
    pub return_type: Option<UnresolvedTypeReference>,
    pub argument_types: Vec<ObjectTypeArgument>,
}

impl NamespaceTypeFunction {
    pub fn ffi_name(&self) -> String {
        let mut nm = String::from("fxa_"); // XXX TODO: calculate prefix from the containing Object declaration, somehow.
        nm.push_str(&self.name);
        nm
    }

    pub fn rust_name(&self) -> String {
        self.name.to_string()
    }
}

impl TryFrom<&weedle::namespace::OperationNamespaceMember<'_>> for NamespaceTypeFunction {
    type Error = anyhow::Error;
    fn try_from(f: &weedle::namespace::OperationNamespaceMember) -> Result<Self> {
        if f.attributes.is_some() {
            bail!("no interface member attributes supported yet");
        }
        if let None = f.identifier {
            bail!("anonymous functions are not supported {:?}", f);
        }
        Ok(NamespaceTypeFunction {
            name: f.identifier.unwrap().0.to_string(),
            return_type: match &f.return_type {
                weedle::types::ReturnType::Void(_) => None,
                weedle::types::ReturnType::Type(t) => Some(t.try_into()?)
            },
            argument_types: try_into_collection!(f.args.body.list)?
        })
    }
}

// Represents a simple C-style enum.
// In the FFI these are turned into an appropriately-sized unsigned integer.
#[derive(Debug, Default)]
pub struct EnumType {
    pub name: String,
    pub values: Vec<String>,
}

impl EnumType {
    pub fn rust_name(&self) -> String {
        self.name.to_string()
    }
}

impl TryFrom<&weedle::EnumDefinition<'_>> for EnumType {
    type Error = anyhow::Error;
    fn try_from(d: &weedle::EnumDefinition) -> Result<Self> {
        if d.attributes.is_some() {
            bail!("no enum attributes are supported yet");
        }
        Ok(EnumType {
            name: d.identifier.0.to_string(),
            values: d.values.body.list.iter().map(|v| v.0.to_string()).collect(),
        })
    }
}


// Represents a type, either primitive or compound.
#[derive(Debug)]
pub enum TypeReference<'a> {
    Boolean,
    String,
    U8,
    S8,
    U16,
    S16,
    U32,
    S32,
    U64,
    S64,
    Object(&'a ObjectType),
    Record(&'a RecordType),
    Enum(&'a EnumType),
    Sequence(Box<TypeReference<'a>>),
    //Union(Vec<Box<TypeReference<'a>>>),
}

// Represents a to-be-resolved reference to a type, either primitive or compound.
// We can't find the actual type until after the whole file has been parsed, so we
// use `UnresolvedTypeReference` as a placeholder and figure out the concrete type
// in post-processing.
#[derive(Debug)]
pub enum UnresolvedTypeReference {
    Boolean,
    Sequence(Box<UnresolvedTypeReference>), // XXX TODO: these boxes could probably just be references
    //Union(Vec<Box<UnresolvedTypeReference>>),
    ByName(String),
}

impl UnresolvedTypeReference {
    pub fn resolve<'a>(&'a self, ci: &'a ComponentInterface) -> Result<TypeReference<'a>> {
        Ok(match self {
            UnresolvedTypeReference::Boolean => TypeReference::Boolean,
            UnresolvedTypeReference::Sequence(t) => TypeReference::Sequence(Box::new(t.resolve(ci)?)),
            //UnresolvedTypeReference::Union(ts) => TypeReference::Union(ts.iter().map(|t| t.resolve(ci)).collect::<Result<Vec<Box<_>>, anyhow::Error>>()?),
            UnresolvedTypeReference::ByName(name) => {
                // Hard-code a couple of our own non-WebIDL-standard type names.
                match name.as_ref() {
                    // XXX TODO what if someone typedefs one of these standard names?
                    // Detect it and throw an error?
                    "string" => TypeReference::String,
                    "u8" => TypeReference::U8,
                    "s8" => TypeReference::S8,
                    "u16" => TypeReference::U16,
                    "s16" => TypeReference::S16,
                    "u32" => TypeReference::U32,
                    "s32" => TypeReference::S32,
                    "u64" => TypeReference::U64,
                    "s64" => TypeReference::S64,
                    _ => {
                        self.resolve_by_name(ci, name)?
                    },
                }
            }
        })
    }

    fn resolve_by_name<'a>(&'a self, ci: &'a ComponentInterface, name: &'a str) -> Result<TypeReference<'a>> {
        // XXX TODO: this is dumb, the ComponentInterface should have a HashMap of type names.
        ci.members.iter().find_map(|m| {
            match m {
                InterfaceMember::Object(obj) => {
                    if obj.name == name { Some(TypeReference::Object(obj)) } else { None }
                },
                InterfaceMember::Record(rec) => {
                    if rec.name == name { Some(TypeReference::Record(rec)) } else { None }
                },
                InterfaceMember::Enum(e) => {
                    if e.name == name { Some(TypeReference::Enum(e)) } else { None }
                },
                _ => None
            }
        }).ok_or_else(|| { anyhow!("unknown type name: {}", &name ) })
    }
}

impl TryFrom<&weedle::types::Type<'_>> for UnresolvedTypeReference {
    type Error = anyhow::Error;
    fn try_from(t: &weedle::types::Type) -> Result<Self> {
        Ok(match t {
            weedle::types::Type::Single(t) => {
                match t {
                    weedle::types::SingleType::Any(_) => bail!("no support for `any` types"),
                    weedle::types::SingleType::NonAny(t) => t.try_into()?,
                }
            },
            weedle::types::Type::Union(t) => {
                bail!("no support for union types yet")
/*                if t.q_mark.is_some() {
                    ;
                }
                UnresolvedTypeReference::Union(t.type_.body.list.iter().map(|v| Box::new(match v {
                    weedle::types::UnionMemberType::Single(t) => {
                        t.try_into().unwrap()
                    },
                    weedle::types::UnionMemberType::Union(t) => panic!("no support for union union member types yet"),
              })).collect())*/
            },
        })
    }
}

impl TryFrom<weedle::types::NonAnyType<'_>> for UnresolvedTypeReference {
    type Error = anyhow::Error;
    fn try_from(t: weedle::types::NonAnyType) -> Result<Self> {
        (&t).try_into()
    }
}

impl TryFrom<&weedle::types::NonAnyType<'_>> for UnresolvedTypeReference {
    type Error = anyhow::Error;
    fn try_from(t: &weedle::types::NonAnyType) -> Result<Self> {
        Ok(match t {
            weedle::types::NonAnyType::Boolean(t) => t.try_into()?,
            weedle::types::NonAnyType::Identifier(t) => t.try_into()?,
            weedle::types::NonAnyType::Integer(t) => t.try_into()?,
            weedle::types::NonAnyType::Sequence(t) => t.try_into()?,
            _ => bail!("no support for type reference {:?}", t),
        })
    }
}

impl TryFrom<&weedle::types::AttributedNonAnyType<'_>> for UnresolvedTypeReference {
    type Error = anyhow::Error;
    fn try_from(t: &weedle::types::AttributedNonAnyType) -> Result<Self> {
        if t.attributes.is_some() {
            bail!("type attributes no support yet");
        }
        (&t.type_).try_into()
    }
}

impl TryFrom<&weedle::types::AttributedType<'_>> for UnresolvedTypeReference {
    type Error = anyhow::Error;
    fn try_from(t: &weedle::types::AttributedType) -> Result<Self> {
        if t.attributes.is_some() {
            bail!("type attributes no support yet");
        }
        (&t.type_).try_into()
    }
}

// The `Clone` bound here is because I don't know enough about the typesystem
// to know of to make this generic over T when T has lifetimes involved.
impl <T: TryInto<UnresolvedTypeReference, Error=anyhow::Error> + Clone> TryFrom<&weedle::types::MayBeNull<T>> for UnresolvedTypeReference {
    type Error = anyhow::Error;
    fn try_from(t: &weedle::types::MayBeNull<T>) -> Result<Self> {
        if t.q_mark.is_some() {
            bail!("no support for nullable types yet");
        }
        TryInto::try_into(t.type_.clone())
    }
}

impl TryFrom<weedle::types::IntegerType> for UnresolvedTypeReference {
    type Error = anyhow::Error;
    fn try_from(t: weedle::types::IntegerType) -> Result<Self> {
        bail!("integer types not implemented ({:?}); consider using u8, u16, u32 or u64", t)
    }
}

impl TryFrom<weedle::term::Boolean> for UnresolvedTypeReference {
    type Error = anyhow::Error;
    fn try_from(t: weedle::term::Boolean) -> Result<Self> {
        Ok(UnresolvedTypeReference::Boolean)
    }
}

impl TryFrom<weedle::types::SequenceType<'_>> for UnresolvedTypeReference {
    type Error = anyhow::Error;
    fn try_from(t: weedle::types::SequenceType) -> Result<Self> {
        Ok(UnresolvedTypeReference::Sequence(Box::new(t.generics.body.as_ref().try_into()?)))
    }
}

impl TryFrom<weedle::common::Identifier<'_>> for UnresolvedTypeReference {
    type Error = anyhow::Error;
    fn try_from(t: weedle::common::Identifier) -> Result<Self> {
        Ok(UnresolvedTypeReference::ByName(t.0.to_string()))
    }
}

// Represents a literal value.
// Used for e.g. default argument values.
#[derive(Debug)]
pub enum Literal {
    Boolean(bool),
    String(String),
    // TODO: more types of literal
}

impl TryFrom<weedle::literal::DefaultValue<'_>> for Literal {
    type Error = anyhow::Error;
    fn try_from(v: weedle::literal::DefaultValue) -> Result<Self> {
        Ok(match v {
            weedle::literal::DefaultValue::Boolean(b) => Literal::Boolean(b.0),
            weedle::literal::DefaultValue::String(s) => Literal::String(s.0.to_string()),
            _ => bail!("no support for {:?} literal yet", v),
        })
    }
}
