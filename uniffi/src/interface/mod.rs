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
//! of the interface provided by a component, in two parts:
//!
//!    * The high-level consumer API, interms of objects and records and methods and so-on
//!    * The low-level FFI contract through which the host language can call into Rust.
//!
//! That's really the key concept of this crate so it's worth repeating: a `ComponentInterface` completely
//! defines the shape and semantics of an interface between the rust-based implementation of a component
//! and the foreign language consumers, including
//! details like:
//!
//!    * The names of all symbols in the compiled object file
//!    * The type and arity of all exported functions
//!    * The layout and conventions used for all arguments and return types
//!
//! If you have a dynamic library compiled from a rust component using this crate, and a foreign
//! language binding generated from the same `ComponentInterface` using the same version of this
//! crate, then there should be no opportunities for them to disagree on how they should interact.
//!
//! General and incomplete TODO list for this thing:
//!
//!   * It should prevent user error and the possibility of generating bad code by doing (at least)
//!     the following checks:
//!       * No duplicate names (types, methods, args, etc)
//!       * No Shadowing of builtin names, or names we use in code generation
//!     We expect that if the user actually does one of these things, then they *should* get a compile
//!     error when trying to build the component, because the codegen will be invalid. But we can't
//!     guarantee that there's not some edge-case where it produces valid-but-incorrect code.
//!
//!   * There is a *lot* of cloning going on, in the spirit of "first make it work". There's probably
//!     a good opportunity here for e.g. interned strings, but we're nowhere near the point were we need
//!     that kind of optimization just yet.
//!
//!   * Error messages leave a lot to be desired (and we currently panc on errors parsing the WebIDL).
//!     If this were to be a real thing we'd need to invest in more empathetic error messages.

use std::{collections::hash_map::Entry, collections::HashMap, convert::TryFrom, str::FromStr};

use anyhow::bail;
use anyhow::Result;
use serde::{Deserialize, Serialize};

pub mod types;
pub use types::TypeReference;
use types::TypeResolver;

/// The main public interface for this module, representing the complete details of an interface exposed
/// by a rust component and the details of consuming it via an extern-C FFI layer.
///
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ComponentInterface {
    /// Every ComponentInterface gets tagged with the version of uniffi used to create it.
    /// This helps us avoid using a lib compiled with one version together with bindings created
    /// using a different version, which might introduce unsafety.
    uniffi_version: String,
    /// A map of all the nameable types used in the interface (including type aliases)
    types: HashMap<String, TypeReference>,
    namespace: String,
    /// The high-level API provided by the component.
    enums: Vec<Enum>,
    records: Vec<Record>,
    functions: Vec<Function>,
    objects: Vec<Object>,
}

impl<'ci> ComponentInterface {
    /// Parse a `ComponentInterface` from a string containing a WebIDL definition.
    pub fn from_webidl(idl: &str) -> Result<Self> {
        let mut ci = Self::default();
        ci.uniffi_version = env!("CARGO_PKG_VERSION").to_string();
        // There's some lifetime thing with the errors returned from weedle::parse
        // that life is too short to figure out; unwrap and move on.
        let defns = weedle::parse(idl.trim()).unwrap();
        // We process the WebIDL definitions in two passes.
        // First, go through and look for all the named types.
        types::TypeFinder::find_type_definitions(&defns, &mut ci)?;
        // With those names resolved, we can build a complete representation of the API.
        APIBuilder::process(&defns, &mut ci)?;
        if ci.namespace.is_empty() {
            bail!("missing namespace definition");
        }
        // Now that the high-level API is settled, we can derive the low-level FFI.
        ci.derive_ffi_funcs()?;
        Ok(ci)
    }

    pub fn from_bincode(data: &[u8]) -> Result<Self> {
        match bincode::deserialize::<ComponentInterface>(data) {
            Err(_) => bail!("Unable to deserialize ComponentInterface; maybe this data is from a different version of uniffi?"),
            Ok(ci) => {
                if ci.uniffi_version !=env!("CARGO_PKG_VERSION") {
                    bail!("It's not safe to use a ComponentInterface built with a different version of uniffi (this is v{}, interface is from v{})", env!("CARGO_PKG_VERSION"), ci.uniffi_version);
                }
                Ok(ci)
            }
        }
    }

    pub fn to_bincode(&self) -> Vec<u8> {
        // Serialization of this struct is infallible.
        bincode::serialize(self).unwrap()
    }

    pub fn namespace(&self) -> &str {
        self.namespace.as_str()
    }

    pub fn iter_enum_definitions(&self) -> Vec<Enum> {
        self.enums.to_vec()
    }

    pub fn iter_record_definitions(&self) -> Vec<Record> {
        self.records.to_vec()
    }

    pub fn iter_function_definitions(&self) -> Vec<Function> {
        self.functions.to_vec()
    }

    pub fn iter_object_definitions(&self) -> Vec<Object> {
        self.objects.to_vec()
    }

    pub fn ffi_bytebuffer_alloc(&self) -> FFIFunction {
        FFIFunction {
            name: format!("{}_bytebuffer_alloc", self.namespace()),
            arguments: vec![Argument {
                name: "size".to_string(),
                type_: TypeReference::U32,
                optional: false,
                default: None,
            }],
            return_type: Some(TypeReference::Bytes),
        }
    }

    pub fn ffi_bytebuffer_free(&self) -> FFIFunction {
        FFIFunction {
            name: format!("{}_bytebuffer_free", self.namespace()),
            arguments: vec![Argument {
                name: "buf".to_string(),
                type_: TypeReference::Bytes,
                optional: false,
                default: None,
            }],
            return_type: None,
        }
    }

    pub fn ffi_string_free(&self) -> FFIFunction {
        FFIFunction {
            name: format!("{}_string_free", self.namespace()),
            arguments: vec![Argument {
                name: "str".to_string(),
                type_: TypeReference::RawStringPointer,
                optional: false,
                default: None,
            }],
            return_type: None,
        }
    }

    pub fn iter_ffi_function_definitions(&self) -> Vec<FFIFunction> {
        self.objects
            .iter()
            .map(|obj| {
                obj.constructors
                    .iter()
                    .map(|f| f.ffi_func.clone())
                    .chain(obj.methods.iter().map(|f| f.ffi_func.clone()))
            })
            .flatten()
            .chain(self.functions.iter().map(|f| f.ffi_func.clone()))
            .chain(
                vec![
                    self.ffi_bytebuffer_alloc(),
                    self.ffi_bytebuffer_free(),
                    self.ffi_string_free(),
                ]
                .iter()
                .cloned(),
            )
            .collect()
    }

    //
    // Private methods for building a ComponentInterface.
    //

    fn get_type_definition(&self, name: &str) -> Option<TypeReference> {
        self.types.get(name).cloned()
    }

    fn add_type_definition(&mut self, name: &str, type_: TypeReference) -> Result<()> {
        match self.types.entry(name.to_string()) {
            Entry::Occupied(_) => bail!("Conflicting type definition for {}", name),
            Entry::Vacant(e) => {
                e.insert(type_);
                Ok(())
            }
        }
    }

    fn add_namespace_definition(&mut self, defn: Namespace) -> Result<()> {
        if !self.namespace.is_empty() {
            bail!("duplicate namespace definition");
        }
        self.namespace.push_str(&defn.name);
        Ok(())
    }

    fn add_enum_definition(&mut self, defn: Enum) -> Result<()> {
        // XXX TODO: reject duplicates? there shouldn't be any thanks to type-finding pass.
        self.enums.push(defn);
        Ok(())
    }

    fn add_record_definition(&mut self, defn: Record) -> Result<()> {
        // XXX TODO: reject duplicates? there shouldn't be any thanks to type-finding pass.
        self.records.push(defn);
        Ok(())
    }

    fn add_function_definition(&mut self, defn: Function) -> Result<()> {
        // XXX TODO: reject duplicates.
        self.functions.push(defn);
        Ok(())
    }

    fn add_object_definition(&mut self, defn: Object) -> Result<()> {
        // XXX TODO: reject duplicates? there shouldn't be any thanks to type-finding pass.
        self.objects.push(defn);
        Ok(())
    }

    fn derive_ffi_funcs(&mut self) -> Result<()> {
        let ci_prefix = self.namespace().to_string();
        for func in self.functions.iter_mut() {
            func.derive_ffi_func(&ci_prefix)?;
        }
        for obj in self.objects.iter_mut() {
            obj.derive_ffi_funcs(&ci_prefix)?;
        }
        Ok(())
    }
}

impl FromStr for ComponentInterface {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self> {
        ComponentInterface::from_webidl(s)
    }
}

trait APIBuilder {
    fn process(&self, ci: &mut ComponentInterface) -> Result<()>;
}

trait APIConverter<T> {
    fn convert(&self, ci: &ComponentInterface) -> Result<T>;
}

impl<T: APIBuilder> APIBuilder for Vec<T> {
    fn process(&self, ci: &mut ComponentInterface) -> Result<()> {
        for item in self.iter() {
            item.process(ci)?;
        }
        Ok(())
    }
}

impl<U, T: APIConverter<U>> APIConverter<Vec<U>> for Vec<T> {
    fn convert(&self, ci: &ComponentInterface) -> Result<Vec<U>> {
        self.iter().map(|v| v.convert(ci)).collect::<Result<_>>()
    }
}

impl APIBuilder for weedle::Definition<'_> {
    fn process(&self, ci: &mut ComponentInterface) -> Result<()> {
        match self {
            weedle::Definition::Namespace(d) => d.process(ci),
            weedle::Definition::Enum(d) => ci.add_enum_definition(d.convert(ci)?),
            weedle::Definition::Dictionary(d) => ci.add_record_definition(d.convert(ci)?),
            weedle::Definition::Interface(d) => ci.add_object_definition(d.convert(ci)?),
            _ => bail!("don't know how to deal with {:?}", self),
        }
    }
}

/// A namespace is currently just a name, but might hold more metadata about
/// the component in future.
///
/// In WebIDL, each `namespace` declares a set of functions and attriutes that
/// are exposed as a global object, and there can be any number of such definitions.
///
/// For our purposes, we expect just a single `namespace` declaration, which defines
/// properties of the component as a whole. It can contain functions but these will
/// be exposed as individual plain functions on the component.
///
/// Yeah, this is a bit of mis-match between WebIDL and our notion of a component,
/// but it's close enough to get us up and running for now.
///
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Namespace {
    name: String,
}

impl APIBuilder for weedle::NamespaceDefinition<'_> {
    fn process(&self, ci: &mut ComponentInterface) -> Result<()> {
        if self.attributes.is_some() {
            bail!("namespace attributes are not supported yet");
        }
        ci.add_namespace_definition(Namespace {
            name: self.identifier.0.to_string(),
        })?;
        for func in self.members.body.convert(ci)? {
            ci.add_function_definition(func)?;
        }
        Ok(())
    }
}

// Represents a standalone function.
//
// The in FFI, this will be a standalone function.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Function {
    name: String,
    arguments: Vec<Argument>,
    return_type: Option<TypeReference>,
    ffi_func: FFIFunction,
}

impl Function {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn arguments(&self) -> Vec<&Argument> {
        self.arguments.iter().collect()
    }
    pub fn return_type(&self) -> Option<&TypeReference> {
        self.return_type.as_ref()
    }
    pub fn ffi_func(&self) -> &FFIFunction {
        &self.ffi_func
    }

    fn derive_ffi_func(&mut self, ci_prefix: &str) -> Result<()> {
        self.ffi_func.name.push_str(ci_prefix);
        self.ffi_func.name.push_str("_");
        self.ffi_func.name.push_str(&self.name);
        self.ffi_func.arguments = self.arguments.clone();
        self.ffi_func.return_type = self.return_type.clone();
        Ok(())
    }
}

// Represents an argument to a function/constructor/method call.
//
// Each argument has a name and a type, along with some optional metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Argument {
    name: String,
    type_: TypeReference,
    optional: bool,
    default: Option<Literal>,
}

impl Argument {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn type_(&self) -> TypeReference {
        self.type_.clone()
    }
}

// Represents an "extern C"-style function that will be part of the FFI.

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct FFIFunction {
    name: String,
    arguments: Vec<Argument>,
    return_type: Option<TypeReference>,
}

impl FFIFunction {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn arguments(&self) -> Vec<&Argument> {
        self.arguments.iter().collect()
    }
    pub fn return_type(&self) -> Option<&TypeReference> {
        self.return_type.as_ref()
    }
}

impl APIConverter<Function> for weedle::namespace::NamespaceMember<'_> {
    fn convert(&self, ci: &ComponentInterface) -> Result<Function> {
        match self {
            weedle::namespace::NamespaceMember::Operation(f) => f.convert(ci),
            _ => bail!("no support for namespace member type {:?} yet", self),
        }
    }
}

impl APIConverter<Function> for weedle::namespace::OperationNamespaceMember<'_> {
    fn convert(&self, ci: &ComponentInterface) -> Result<Function> {
        if self.attributes.is_some() {
            bail!("no interface member attributes supported yet");
        }
        Ok(Function {
            name: match self.identifier {
                None => bail!("anonymous functions are not supported {:?}", self),
                Some(id) => id.0.to_string(),
            },
            return_type: match &self.return_type {
                weedle::types::ReturnType::Void(_) => None,
                weedle::types::ReturnType::Type(t) => Some(t.resolve_type_definition(ci)?),
            },
            arguments: self.args.body.list.convert(ci)?,
            ffi_func: Default::default(),
        })
    }
}

impl APIConverter<Argument> for weedle::argument::Argument<'_> {
    fn convert(&self, ci: &ComponentInterface) -> Result<Argument> {
        match self {
            weedle::argument::Argument::Single(t) => t.convert(ci),
            weedle::argument::Argument::Variadic(_) => bail!("variadic arguments not supported"),
        }
    }
}

impl APIConverter<Argument> for weedle::argument::SingleArgument<'_> {
    fn convert(&self, ci: &ComponentInterface) -> Result<Argument> {
        if self.attributes.is_some() {
            bail!("argument attributes are not supported yet");
        }
        Ok(Argument {
            name: self.identifier.0.to_string(),
            type_: (&self.type_).resolve_type_definition(ci)?,
            optional: self.optional.is_some(),
            default: match self.default {
                None => None,
                Some(v) => Some(v.value.convert(ci)?),
            },
        })
    }
}

// Represents a simple C-style enum.
// In the FFI these are turned into a simple u32.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Enum {
    name: String,
    values: Vec<String>,
}

impl Enum {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn values(&self) -> Vec<&str> {
        self.values.iter().map(|v| v.as_str()).collect()
    }
}

impl APIConverter<Enum> for weedle::EnumDefinition<'_> {
    fn convert(&self, _ci: &ComponentInterface) -> Result<Enum> {
        if self.attributes.is_some() {
            bail!("enum attributes are not supported yet");
        }
        Ok(Enum {
            name: self.identifier.0.to_string(),
            values: self
                .values
                .body
                .list
                .iter()
                .map(|v| v.0.to_string())
                .collect(),
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
///  - maybe "Class" would be a better name than "Object" here?
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Object {
    name: String,
    constructors: Vec<Constructor>,
    methods: Vec<Method>,
}

impl Object {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn constructors(&self) -> Vec<&Constructor> {
        self.constructors.iter().collect()
    }

    pub fn methods(&self) -> Vec<&Method> {
        self.methods.iter().collect()
    }

    fn derive_ffi_funcs(&mut self, ci_prefix: &str) -> Result<()> {
        for cons in self.constructors.iter_mut() {
            cons.derive_ffi_func(ci_prefix, &self.name)?
        }
        for meth in self.methods.iter_mut() {
            meth.derive_ffi_func(ci_prefix, &self.name)?
        }
        Ok(())
    }
}

// Represents a constructor for an object type.
//
// In the FFI, this will be a function that returns a handle for an instance
// of the corresponding object type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constructor {
    name: String,
    arguments: Vec<Argument>,
    ffi_func: FFIFunction,
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

    fn derive_ffi_func(&mut self, ci_prefix: &str, obj_prefix: &str) -> Result<()> {
        self.ffi_func.name.push_str(ci_prefix);
        self.ffi_func.name.push_str("_");
        self.ffi_func.name.push_str(obj_prefix);
        self.ffi_func.name.push_str("_");
        self.ffi_func.name.push_str(&self.name);
        self.ffi_func.arguments = self.arguments.clone();
        self.ffi_func.return_type = Some(TypeReference::Object(obj_prefix.to_string()));
        Ok(())
    }
}

// Represents an instance method for an object type.
//
// The in FFI, this will be a function whose first argument is a handle for an
// instance of the corresponding object type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Method {
    name: String,
    return_type: Option<TypeReference>,
    arguments: Vec<Argument>,
    ffi_func: FFIFunction,
}

impl Method {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn arguments(&self) -> Vec<&Argument> {
        self.arguments.iter().collect()
    }

    pub fn return_type(&self) -> Option<&TypeReference> {
        self.return_type.as_ref()
    }

    pub fn ffi_func(&self) -> &FFIFunction {
        &self.ffi_func
    }

    pub fn first_argument(&self) -> Argument {
        Argument {
            name: "handle".to_string(),
            type_: TypeReference::Object(self.name.clone()),
            optional: false,
            default: None,
        }
    }

    fn derive_ffi_func(&mut self, ci_prefix: &str, obj_prefix: &str) -> Result<()> {
        self.ffi_func.name.push_str(ci_prefix);
        self.ffi_func.name.push_str("_");
        self.ffi_func.name.push_str(obj_prefix);
        self.ffi_func.name.push_str("_");
        self.ffi_func.name.push_str(&self.name);
        self.ffi_func.arguments = vec![self.first_argument()]
            .iter()
            .cloned()
            .chain(self.arguments.iter().cloned())
            .collect();
        self.ffi_func.return_type = self.return_type.clone();
        Ok(())
    }
}

impl APIConverter<Object> for weedle::InterfaceDefinition<'_> {
    fn convert(&self, ci: &ComponentInterface) -> Result<Object> {
        if self.attributes.is_some() {
            bail!("interface attributes are not supported yet");
        }
        if self.inheritance.is_some() {
            bail!("interface inheritence is not supported");
        }
        let mut object = Object {
            name: self.identifier.0.to_string(),
            constructors: Default::default(),
            methods: Default::default(),
        };
        for member in &self.members.body {
            match member {
                weedle::interface::InterfaceMember::Constructor(t) => {
                    object.constructors.push(t.convert(ci)?);
                }
                weedle::interface::InterfaceMember::Operation(t) => {
                    object.methods.push(t.convert(ci)?);
                }
                _ => bail!("no support for interface member type {:?} yet", member),
            }
        }
        Ok(object)
    }
}

impl APIConverter<Constructor> for weedle::interface::ConstructorInterfaceMember<'_> {
    fn convert(&self, ci: &ComponentInterface) -> Result<Constructor> {
        if self.attributes.is_some() {
            bail!("constructor attributes are not supported yet");
        }
        Ok(Constructor {
            name: String::from("new"), // TODO: get the name from an attribute maybe?
            arguments: self.args.body.list.convert(ci)?,
            ffi_func: Default::default(),
        })
    }
}

impl APIConverter<Method> for weedle::interface::OperationInterfaceMember<'_> {
    fn convert(&self, ci: &ComponentInterface) -> Result<Method> {
        if self.attributes.is_some() {
            bail!("no interface member attributes supported yet");
        }
        if self.special.is_some() {
            bail!("special operations not supported");
        }
        if let Some(weedle::interface::StringifierOrStatic::Stringifier(_)) = self.modifier {
            bail!("stringifiers are not supported");
        }
        Ok(Method {
            name: match self.identifier {
                None => bail!("anonymous methods are not supported {:?}", self),
                Some(id) => id.0.to_string(),
            },
            arguments: self.args.body.list.convert(ci)?,
            return_type: match &self.return_type {
                weedle::types::ReturnType::Void(_) => None,
                weedle::types::ReturnType::Type(t) => Some(t.resolve_type_definition(ci)?),
            },
            ffi_func: Default::default(),
        })
    }
}

// Represents a "data class" style object, for passing around complex values.
// In the FFI these are represented as a ByteBuffer, which one side explicitly
// serializes the data into and the other serializes it out of. So I guess they're
// kind of like "pass by clone" values.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Record {
    name: String,
    fields: Vec<Field>,
}

impl Record {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn fields(&self) -> Vec<&Field> {
        self.fields.iter().collect()
    }
}
// Represents an individual field on a Record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Field {
    name: String,
    type_: TypeReference,
    required: bool,
    default: Option<Literal>,
}
impl Field {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn type_(&self) -> TypeReference {
        self.type_.clone()
    }
}

impl APIConverter<Record> for weedle::DictionaryDefinition<'_> {
    fn convert(&self, ci: &ComponentInterface) -> Result<Record> {
        if self.attributes.is_some() {
            bail!("dictionary attributes are not supported yet");
        }
        if self.inheritance.is_some() {
            bail!("dictionary inheritence is not supported");
        }
        Ok(Record {
            name: self.identifier.0.to_string(),
            fields: self.members.body.convert(ci)?,
        })
    }
}

impl APIConverter<Field> for weedle::dictionary::DictionaryMember<'_> {
    fn convert(&self, ci: &ComponentInterface) -> Result<Field> {
        if self.attributes.is_some() {
            bail!("dictionary member attributes are not supported yet");
        }
        Ok(Field {
            name: self.identifier.0.to_string(),
            type_: (&self.type_).resolve_type_definition(ci)?,
            required: self.required.is_some(),
            default: match self.default {
                None => None,
                Some(v) => Some(v.value.convert(ci)?),
            },
        })
    }
}

// Represents a literal value.
// Used for e.g. default argument values.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Literal {
    Boolean(bool),
    String(String),
    // TODO: more types of literal
}

impl APIConverter<Literal> for weedle::literal::DefaultValue<'_> {
    fn convert(&self, _ci: &ComponentInterface) -> Result<Literal> {
        Ok(match self {
            weedle::literal::DefaultValue::Boolean(b) => Literal::Boolean(b.0),
            weedle::literal::DefaultValue::String(s) => Literal::String(s.0.to_string()),
            _ => bail!("no support for {:?} literal yet", self),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Attribute {
    // Add valid attributes here
}

impl TryFrom<&weedle::attribute::ExtendedAttribute<'_>> for Attribute {
    type Error = anyhow::Error;
    fn try_from(
        weedle_attribute: &weedle::attribute::ExtendedAttribute,
    ) -> Result<Self, anyhow::Error> {
        match weedle_attribute {
            // Add attribute conversions here:
            _ => anyhow::bail!("Attribute not supported: {:?}", weedle_attribute),
        }
    }
}

/// Abstraction around a Vec<Attribute>. Used to define conversions between a
/// weedle list of attributes and itself.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attributes(Vec<Attribute>);

impl TryFrom<&weedle::attribute::ExtendedAttributeList<'_>> for Attributes {
    type Error = anyhow::Error;
    fn try_from(
        weedle_attributes: &weedle::attribute::ExtendedAttributeList,
    ) -> Result<Self, Self::Error> {
        // TODO: Error out on duplicate attributes
        weedle_attributes
            .body
            .list
            .iter()
            .map(|attr| Attribute::try_from(attr))
            .collect::<Result<Vec<_>, _>>()
            .map(|attrs| Attributes(attrs))
    }
}
