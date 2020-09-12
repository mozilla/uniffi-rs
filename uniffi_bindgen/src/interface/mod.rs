/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! # Component Interface Definition and Types
//!
//! This crate provides an abstract representation of the interface provided by a uniffi rust component,
//! in high-level terms suitable for translation into target consumer languages such as Kotlin
//! and Swift. It also provides facilities for parsing a WebIDL interface definition file into such a
//! representation.
//!
//! The entrypoint to this crate is the `ComponentInterface` struct, which holds a complete definition
//! of the interface provided by a component, in two parts:
//!
//!    * The high-level consumer API, in terms of objects and records and methods and so-on
//!    * The low-level FFI contract through which the foreign language code can call into Rust.
//!
//! That's really the key concept of this crate so it's worth repeating: a `ComponentInterface` completely
//! defines the shape and semantics of an interface between the rust-based implementation of a component
//! and its foreign language consumers, including details like:
//!
//!    * The names of all symbols in the compiled object file
//!    * The type and arity of all exported functions
//!    * The layout and conventions used for all arguments and return types
//!
//! If you have a dynamic library compiled from a rust component using this crate, and a foreign
//! language binding generated from the same `ComponentInterface` using the same version of this
//! crate, then there should be no opportunities for them to disagree on how the two sides should
//! interact.
//!
//! General and incomplete TODO list for this thing:
//!
//!   * It should prevent user error and the possibility of generating bad code by doing (at least)
//!     the following checks:
//!       * No duplicate names (types, methods, args, etc)
//!       * No shadowing of builtin names, or names we use in code generation
//!     We expect that if the user actually does one of these things, then they *should* get a compile
//!     error when trying to build the component, because the codegen will be invalid. But we can't
//!     guarantee that there's not some edge-case where it produces valid-but-incorrect code.
//!
//!   * There is a *lot* of cloning going on, in the spirit of "first make it work". There's probably
//!     a good opportunity here for e.g. interned strings, but we're nowhere near the point were we need
//!     that kind of optimization just yet.
//!
//!   * Error messages and general developer experience leave a lot to be desired.

use std::{
    collections::hash_map::{DefaultHasher, Entry},
    collections::HashMap,
    convert::TryFrom,
    hash::{Hash, Hasher},
    str::FromStr,
};

use anyhow::bail;
use anyhow::Result;
use serde::{Deserialize, Serialize};

pub mod types;
use types::TypeResolver;
pub use types::{FFIType, Type};

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
    types: HashMap<String, Type>,
    namespace: String,
    /// The high-level API provided by the component.
    enums: Vec<Enum>,
    records: Vec<Record>,
    functions: Vec<Function>,
    objects: Vec<Object>,
    errors: Vec<Error>,
}

impl<'ci> ComponentInterface {
    /// Parse a `ComponentInterface` from a string containing a WebIDL definition.
    pub fn from_webidl(idl: &str) -> Result<Self> {
        let mut ci = Self::default();
        ci.uniffi_version = env!("CARGO_PKG_VERSION").to_string();
        // There's some lifetime thing with the errors returned from weedle::parse
        // that my own lifetime is too short to worry about figuring out; unwrap and move on.
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

    pub fn iter_error_definitions(&self) -> Vec<Error> {
        self.errors.to_vec()
    }

    /// Calculate a numeric checksum for this ComponentInterface.
    ///
    /// The checksum can be used to guard against accidentally using foreign-language bindings
    /// generated from one version of an interface with the compiled Rust code from a different
    /// version of that interface. It offers the following properties:
    ///
    ///   - Two ComponentIntefaces generated from the same WebIDL file, using the same version of uniffi
    ///     and the same version of Rust, will always have the same checksum value.
    ///   - Two ComponentInterfaces will, with high probability, have different checksum values if:
    ///         - They were generated from two different WebIDL files.
    ///         - They were generated by two different versions of uniffi
    ///
    /// The checksum may or may not change depending on the version of Rust used; since we expect
    /// consumers to be using the same executable to generate both the scaffolding and the bindings,
    /// assuming the same version of Rust seems acceptable.
    ///
    /// Note that this is designed to prevent accidents, not attacks, so there is no need for the
    /// checksum to be cryptographically secure.
    ///
    /// TODO: it's not clear to me if the derivation of `Hash` is actually deterministic enough to
    /// ensure the guarantees above, or if it might be sensitive to e.g. compiler-driven re-ordering
    /// of struct field. Let's see how it goes...
    pub fn checksum(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        // Our implementation of `Hash` mixes in all of the public API of the component,
        // as well as the version string of uniffi.
        self.hash(&mut hasher);
        return hasher.finish();
    }

    /// The namespace to use in FFI-level function definitions.
    ///
    /// The value returned by this method is used as a prefix to namespace all FFI-level functions
    /// used in this ComponentInterface.
    ///
    /// Since these names are an internal implementation detail that is not typically visible to
    /// consumers, we take the opportunity to add an additional safety guard by including a 4-hex-char
    /// checksum in each name. If foreign-language bindings attempt to load and use a version of the
    /// Rust code compiled from a different IDL definition than the one used for the bindings themselves,
    /// then there is a high probability of checksum mismatch and they will fail to link against the
    /// compiled Rust code. The result will be an ugly inscrutable link-time error, but that is a lot
    /// better than triggering potentially arbitrary memory unsafety!
    pub fn ffi_namespace(&self) -> String {
        format!(
            "{}_{:x}",
            self.namespace,
            (self.checksum() & 0x000000000000FFFF) as u16
        )
    }

    /// Builtin FFI function for allocating a new `RustBuffer`.
    /// This is needed so that the foreign language bindings can create buffers in which to pass
    /// complex data types across the FFI.
    pub fn ffi_rustbuffer_alloc(&self) -> FFIFunction {
        FFIFunction {
            name: format!("ffi_{}_rustbuffer_alloc", self.ffi_namespace()),
            arguments: vec![FFIArgument {
                name: "size".to_string(),
                type_: FFIType::Int32,
            }],
            return_type: Some(FFIType::RustBuffer),
        }
    }

    /// Builtin FFI function for copying foreign-owned bytes
    /// This is needed so that the foreign language bindings can create buffers in which to pass
    /// complex data types across the FFI.
    pub fn ffi_rustbuffer_from_bytes(&self) -> FFIFunction {
        FFIFunction {
            name: format!("ffi_{}_rustbuffer_from_bytes", self.ffi_namespace()),
            arguments: vec![FFIArgument {
                name: "bytes".to_string(),
                type_: FFIType::ForeignBytes,
            }],
            return_type: Some(FFIType::RustBuffer),
        }
    }

    /// Builtin FFI function for freeing a `RustBuffer`.
    /// This is needed so that the foreign language bindings can free buffers in which they received
    /// complex data types returned across the FFI.
    pub fn ffi_rustbuffer_free(&self) -> FFIFunction {
        FFIFunction {
            name: format!("ffi_{}_rustbuffer_free", self.ffi_namespace()),
            arguments: vec![FFIArgument {
                name: "buf".to_string(),
                type_: FFIType::RustBuffer,
            }],
            return_type: None,
        }
    }

    /// Builtin FFI function for reserving extra space in a `RustBuffer`.
    /// This is needed so that the foreign language bindings can grow buffers used for passing
    /// complex data types across the FFI.
    pub fn ffi_rustbuffer_reserve(&self) -> FFIFunction {
        FFIFunction {
            name: format!("ffi_{}_rustbuffer_reserve", self.ffi_namespace()),
            arguments: vec![
                FFIArgument {
                    name: "buf".to_string(),
                    type_: FFIType::RustBuffer,
                },
                FFIArgument {
                    name: "additional".to_string(),
                    type_: FFIType::Int32,
                },
            ],
            return_type: Some(FFIType::RustBuffer),
        }
    }

    /// Builtin FFI function for freeing a string.
    /// This is needed for foreign-language code when dealing with errors, so that it can free
    /// the error message string.
    /// TODO: make our error class return the message in a `RustBuffer` so we can free it using
    /// the exisiting bytebuffer-freeing function rather than a special one.
    pub fn ffi_string_free(&self) -> FFIFunction {
        FFIFunction {
            name: format!("ffi_{}_string_free", self.ffi_namespace()),
            arguments: vec![FFIArgument {
                name: "cstr".to_string(),
                type_: FFIType::RustCString,
            }],
            return_type: None,
        }
    }

    pub fn iter_ffi_function_definitions(&self) -> Vec<FFIFunction> {
        self.objects
            .iter()
            .map(|obj| {
                vec![obj.ffi_object_free().clone()]
                    .into_iter()
                    .chain(obj.constructors.iter().map(|f| f.ffi_func.clone()))
                    .chain(obj.methods.iter().map(|f| f.ffi_func.clone()))
            })
            .flatten()
            .chain(self.functions.iter().map(|f| f.ffi_func.clone()))
            .chain(
                vec![
                    self.ffi_rustbuffer_alloc(),
                    self.ffi_rustbuffer_from_bytes(),
                    self.ffi_rustbuffer_free(),
                    self.ffi_rustbuffer_reserve(),
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

    /// Get the high-level type corresponding to a given name, if any.
    fn get_type_definition(&self, name: &str) -> Option<Type> {
        self.types.get(name).cloned()
    }

    /// Get the high-level type corresponding to a given WebIDL type node.
    fn resolve_type_definition<T: TypeResolver>(&self, type_: T) -> Result<Type> {
        TypeResolver::resolve_type_definition(&type_, self)
    }

    /// Add the definition of a named high-level type.
    ///
    /// This will fail if you try to add a name for which an existing type definition exists.
    fn add_type_definition(&mut self, name: &str, type_: Type) -> Result<()> {
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
        // Note that there will be no duplicates thanks to the previous type-finding pass.
        self.enums.push(defn);
        Ok(())
    }

    fn add_record_definition(&mut self, defn: Record) -> Result<()> {
        // Note that there will be no duplicates thanks to the previous type-finding pass.
        self.records.push(defn);
        Ok(())
    }

    fn add_function_definition(&mut self, defn: Function) -> Result<()> {
        // XXX TODO: reject duplicates; the type-finding pass won't help us here.
        self.functions.push(defn);
        Ok(())
    }

    fn add_object_definition(&mut self, defn: Object) -> Result<()> {
        // Note that there will be no duplicates thanks to the previous type-finding pass.
        self.objects.push(defn);
        Ok(())
    }

    fn add_error_definition(&mut self, defn: Error) -> Result<()> {
        self.errors.push(defn);
        Ok(())
    }

    fn derive_ffi_funcs(&mut self) -> Result<()> {
        let ci_prefix = self.ffi_namespace().to_string();
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

impl Hash for ComponentInterface {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // We can't hash `self.types`, but its contents are implied by the other fields
        // anyway, so it's safe to ignore it.
        self.uniffi_version.hash(state);
        self.namespace.hash(state);
        self.enums.hash(state);
        self.records.hash(state);
        self.functions.hash(state);
        self.objects.hash(state);
        self.errors.hash(state);
    }
}

/// Trait to help build a `ComponentInterface` from WedIDL syntax nodes.
///
/// This trait does structural matching on the various weedle AST nodes and
/// uses them to build up the records, enums, objects etc in the provided
/// `ComponentInterface`.
trait APIBuilder {
    fn process(&self, ci: &mut ComponentInterface) -> Result<()>;
}

impl<T: APIBuilder> APIBuilder for Vec<T> {
    fn process(&self, ci: &mut ComponentInterface) -> Result<()> {
        for item in self.iter() {
            item.process(ci)?;
        }
        Ok(())
    }
}

impl APIBuilder for weedle::Definition<'_> {
    fn process(&self, ci: &mut ComponentInterface) -> Result<()> {
        match self {
            weedle::Definition::Namespace(d) => d.process(ci),
            weedle::Definition::Enum(d) => {
                // We check if the enum represents an error...
                let is_error = if let Some(attrs) = &d.attributes {
                    let attributes = Attributes::try_from(attrs)?;
                    attributes.contains_error_attr()
                } else {
                    false
                };
                if is_error {
                    ci.add_error_definition(d.convert(ci)?)
                } else {
                    ci.add_enum_definition(d.convert(ci)?)
                }
            }
            weedle::Definition::Dictionary(d) => ci.add_record_definition(d.convert(ci)?),
            weedle::Definition::Interface(d) => ci.add_object_definition(d.convert(ci)?),
            _ => bail!("don't know how to deal with {:?}", self),
        }
    }
}

/// A namespace is currently just a name, but might hold more metadata about
/// the component in future.
///
/// In WebIDL, each `namespace` declares a set of functions and attributes that
/// are exposed as a global object, and there can be any number of such definitions.
///
/// For our purposes, we expect just a single `namespace` declaration, which defines
/// properties of the component as a whole. It can contain functions but these will
/// be exposed as individual plain functions on the component.
///
/// Yeah, this is a bit of mis-match between WebIDL and our notion of a component,
/// but it's close enough to get us up and running for now.
///
#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
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

/// Trait to help convert WedIDL syntax nodes into `ComponentInterface` objects.
///
/// This trait does structural matching on the various weedle AST nodes and converts
/// them into appropriate structs that we can use to build up the contents of a
/// `ComponentInterface`. It is basically the `TryFrom` trait except that the conversion
/// always happens in the context of a given `ComponentInterface`, which is used for
/// resolving e.g. type definitions.
///
/// The difference between this trait and `APIBuilder` is that `APIConverter` treats the
/// `ComponentInterface` as a read-only data source for resolving types, while `APIBuilder`
/// actually mutates the `ComponentInterface` to add new definitions.
trait APIConverter<T> {
    fn convert(&self, ci: &ComponentInterface) -> Result<T>;
}

impl<U, T: APIConverter<U>> APIConverter<Vec<U>> for Vec<T> {
    fn convert(&self, ci: &ComponentInterface) -> Result<Vec<U>> {
        self.iter().map(|v| v.convert(ci)).collect::<Result<_>>()
    }
}

/// Represents a standalone function.
///
/// Each `Function` corresponds to a standalone function in the rust module,
/// and has a corresponding standalone function in the foreign language bindings.
///
/// In the FFI, this will be a standalone function with appropriately lowered types.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Function {
    name: String,
    arguments: Vec<Argument>,
    return_type: Option<Type>,
    ffi_func: FFIFunction,
    attributes: Attributes,
}

impl Function {
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

    pub fn throws(&self) -> Option<&str> {
        self.attributes.get_throws_err()
    }

    fn derive_ffi_func(&mut self, ci_prefix: &str) -> Result<()> {
        self.ffi_func.name.push_str(ci_prefix);
        self.ffi_func.name.push_str("_");
        self.ffi_func.name.push_str(&self.name);
        self.ffi_func.arguments = self.arguments.iter().map(FFIArgument::from).collect();
        self.ffi_func.return_type = self.return_type.as_ref().map(FFIType::from);
        Ok(())
    }
}

impl Hash for Function {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // We don't include the FFIFunc in the hash calculation, because:
        //  - it is entirely determined by the other fields,
        //    so excluding it is safe.
        //  - its `name` property includes a checksum derived from  the very
        //    hash value we're trying to calculate here, so excluding it
        //    avoids a weird circular depenendency in the calculation.
        self.name.hash(state);
        self.arguments.hash(state);
        self.return_type.hash(state);
        self.attributes.hash(state);
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
        let return_type = match &self.return_type {
            weedle::types::ReturnType::Void(_) => None,
            weedle::types::ReturnType::Type(t) => Some(ci.resolve_type_definition(t)?),
        };
        if let Some(Type::Object(_)) = return_type {
            bail!("Objects cannot currently be returned from functions");
        }
        Ok(Function {
            name: match self.identifier {
                None => bail!("anonymous functions are not supported {:?}", self),
                Some(id) => id.0.to_string(),
            },
            return_type,
            arguments: self.args.body.list.convert(ci)?,
            ffi_func: Default::default(),
            attributes: match &self.attributes {
                Some(attr) => Attributes::try_from(attr)?,
                None => Attributes(Vec::new()),
            },
        })
    }
}

/// Represents an argument to a function/constructor/method call.
///
/// Each argument has a name and a type, along with some optional metadata.
#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
pub struct Argument {
    name: String,
    type_: Type,
    by_ref: bool,
    optional: bool,
    default: Option<Literal>,
}

impl Argument {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn type_(&self) -> Type {
        self.type_.clone()
    }
    pub fn by_ref(&self) -> bool {
        self.by_ref
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
        let type_ = (&self.type_).resolve_type_definition(ci)?;
        if let Type::Object(_) = type_ {
            bail!("Objects cannot currently be passed as arguments");
        }
        Ok(Argument {
            name: self.identifier.0.to_string(),
            type_,
            by_ref: match &self.attributes {
                None => false,
                Some(attrs) => Attributes::try_from(attrs)?
                    .0
                    .iter()
                    .any(|attr| match attr {
                        Attribute::ByRef => true,
                        _ => false,
                    }),
            },
            optional: self.optional.is_some(),
            default: match self.default {
                None => None,
                Some(v) => Some(v.value.convert(ci)?),
            },
        })
    }
}

/// Represents a simple C-style enum, with named variants.
///
/// In the FFI these are turned into a simple u32.
#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
pub struct Enum {
    name: String,
    variants: Vec<String>,
}

impl Enum {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn variants(&self) -> Vec<&str> {
        self.variants.iter().map(|v| v.as_str()).collect()
    }
}

impl APIConverter<Enum> for weedle::EnumDefinition<'_> {
    fn convert(&self, _ci: &ComponentInterface) -> Result<Enum> {
        Ok(Enum {
            name: self.identifier.0.to_string(),
            variants: self
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
    ffi_func_free: FFIFunction,
}

impl Object {
    fn new(name: String) -> Object {
        Object {
            name,
            constructors: Default::default(),
            methods: Default::default(),
            ffi_func_free: Default::default(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn constructors(&self) -> Vec<&Constructor> {
        self.constructors.iter().collect()
    }

    pub fn methods(&self) -> Vec<&Method> {
        self.methods.iter().collect()
    }

    pub fn ffi_object_free(&self) -> &FFIFunction {
        &self.ffi_func_free
    }

    fn derive_ffi_funcs(&mut self, ci_prefix: &str) -> Result<()> {
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
    fn convert(&self, ci: &ComponentInterface) -> Result<Object> {
        if self.attributes.is_some() {
            bail!("interface attributes are not supported yet");
        }
        if self.inheritance.is_some() {
            bail!("interface inheritence is not supported");
        }
        let mut object = Object::new(self.identifier.0.to_string());
        for member in &self.members.body {
            match member {
                weedle::interface::InterfaceMember::Constructor(t) => {
                    object.constructors.push(t.convert(ci)?);
                }
                weedle::interface::InterfaceMember::Operation(t) => {
                    let mut method = t.convert(ci)?;
                    method.object_name.push_str(object.name.as_str());
                    object.methods.push(method);
                }
                _ => bail!("no support for interface member type {:?} yet", member),
            }
        }
        if object.constructors.is_empty() {
            object.constructors.push(Default::default());
        }
        Ok(object)
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
    attributes: Attributes,
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
        self.ffi_func.arguments = self.arguments.iter().map(FFIArgument::from).collect();
        self.ffi_func.return_type = Some(FFIType::UInt64);
        Ok(())
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
            attributes: Attributes(Vec::new()),
        }
    }
}

impl APIConverter<Constructor> for weedle::interface::ConstructorInterfaceMember<'_> {
    fn convert(&self, ci: &ComponentInterface) -> Result<Constructor> {
        Ok(Constructor {
            name: String::from("new"), // TODO: get the name from an attribute maybe?
            arguments: self.args.body.list.convert(ci)?,
            ffi_func: Default::default(),
            attributes: match &self.attributes {
                Some(attr) => Attributes::try_from(attr)?,
                None => Attributes(Vec::new()),
            },
        })
    }
}

// Represents an instance method for an object type.
//
// The in FFI, this will be a function whose first argument is a handle for an
// instance of the corresponding object type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Method {
    name: String,
    object_name: String,
    return_type: Option<Type>,
    arguments: Vec<Argument>,
    ffi_func: FFIFunction,
    attributes: Attributes,
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
            type_: Type::Object(self.object_name.clone()),
            by_ref: false,
            optional: false,
            default: None,
        }
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
        self.ffi_func.arguments = vec![self.first_argument()]
            .iter()
            .chain(self.arguments.iter())
            .map(FFIArgument::from)
            .collect();
        self.ffi_func.return_type = self.return_type.as_ref().map(FFIType::from);
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
    fn convert(&self, ci: &ComponentInterface) -> Result<Method> {
        if self.special.is_some() {
            bail!("special operations not supported");
        }
        if let Some(weedle::interface::StringifierOrStatic::Stringifier(_)) = self.modifier {
            bail!("stringifiers are not supported");
        }
        let return_type = match &self.return_type {
            weedle::types::ReturnType::Void(_) => None,
            weedle::types::ReturnType::Type(t) => Some(ci.resolve_type_definition(t)?),
        };
        if let Some(Type::Object(_)) = return_type {
            bail!("Objects cannot currently be returned from functions");
        }
        Ok(Method {
            name: match self.identifier {
                None => bail!("anonymous methods are not supported {:?}", self),
                Some(id) => id.0.to_string(),
            },
            // We don't know the name of the containing `Object` at this point, fill it in later.
            object_name: Default::default(),
            arguments: self.args.body.list.convert(ci)?,
            return_type,
            ffi_func: Default::default(),
            attributes: match &self.attributes {
                Some(attr) => Attributes::try_from(attr)?,
                None => Attributes(Vec::new()),
            },
        })
    }
}

/// An error marked in the WebIDL with an [Error]
/// attribute. Used to define exceptions/errors in the bindings
/// as well as defining the From<Error> for ExternError
/// needed for the different errors to cross the FFI.
#[derive(Debug, Clone, Serialize, Deserialize, Default, Hash)]
pub struct Error {
    name: String,
    values: Vec<String>,
}

impl Error {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn values(&self) -> Vec<&str> {
        self.values.iter().map(|v| v.as_str()).collect()
    }
}

impl APIConverter<Error> for weedle::EnumDefinition<'_> {
    fn convert(&self, _ci: &ComponentInterface) -> Result<Error> {
        Ok(Error {
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

/// Represents a "data class" style object, for passing around complex values.
///
/// In the FFI these are represented as a byte buffer, which one side explicitly
/// serializes the data into and the other serializes it out of. So I guess they're
/// kind of like "pass by clone" values.
#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
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

// Represents an individual field on a Record.
#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
pub struct Field {
    name: String,
    type_: Type,
    required: bool,
    default: Option<Literal>,
}

impl Field {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn type_(&self) -> Type {
        self.type_.clone()
    }
}

impl APIConverter<Field> for weedle::dictionary::DictionaryMember<'_> {
    fn convert(&self, ci: &ComponentInterface) -> Result<Field> {
        if self.attributes.is_some() {
            bail!("dictionary member attributes are not supported yet");
        }
        let type_ = ci.resolve_type_definition(&self.type_)?;
        if let Type::Object(_) = type_ {
            bail!("Objects cannot currently appear in record fields");
        }
        Ok(Field {
            name: self.identifier.0.to_string(),
            type_,
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
#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
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

/// Represents an attribute parsed from WebIDL, like [ByRef] or [Throws].
///
/// This is a convenience enum for parsing WebIDL attributes and erroring out if we encounter
/// any unsupported ones. These don't convert directly into parts of a `ComponentInterface`, but
/// may influence the properties of things like functions and arguments.
#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
pub enum Attribute {
    ByRef,
    Throws(String),
    Error,
}

impl Attribute {
    fn is_error(&self) -> bool {
        matches!(self, Attribute::Error)
    }
}

impl TryFrom<&weedle::attribute::ExtendedAttribute<'_>> for Attribute {
    type Error = anyhow::Error;
    fn try_from(
        weedle_attribute: &weedle::attribute::ExtendedAttribute<'_>,
    ) -> Result<Self, anyhow::Error> {
        match weedle_attribute {
            weedle::attribute::ExtendedAttribute::NoArgs(attr) => match (attr.0).0 {
                "ByRef" => Ok(Attribute::ByRef),
                "Error" => Ok(Attribute::Error),
                _ => anyhow::bail!("ExtendedAttributeNoArgs not supported: {:?}", (attr.0).0),
            },
            weedle::attribute::ExtendedAttribute::Ident(identity) => {
                if identity.lhs_identifier.0 == "Throws" {
                    Ok(Attribute::Throws(match identity.rhs {
                        weedle::attribute::IdentifierOrString::Identifier(identifier) => {
                            identifier.0.to_string()
                        }
                        weedle::attribute::IdentifierOrString::String(str_lit) => {
                            str_lit.0.to_string()
                        }
                    }))
                } else {
                    anyhow::bail!(
                        "Attribute identity Identifier not supported: {:?}",
                        identity.lhs_identifier.0
                    )
                }
            }
            _ => anyhow::bail!("Attribute not supported: {:?}", weedle_attribute),
        }
    }
}

/// Abstraction around a Vec<Attribute>.
///
/// This is a convenience for parsing a weedle list of attributes.
#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
pub struct Attributes(Vec<Attribute>);

impl Attributes {
    pub fn contains_error_attr(&self) -> bool {
        self.0.iter().any(|attr| attr.is_error())
    }

    fn get_throws_err(&self) -> Option<&str> {
        self.0.iter().find_map(|attr| match attr {
            // This will hopefully return a helpful compilation error
            // if the error is not defined.
            Attribute::Throws(inner) => Some(inner.as_ref()),
            _ => None,
        })
    }
}

impl TryFrom<&weedle::attribute::ExtendedAttributeList<'_>> for Attributes {
    type Error = anyhow::Error;
    fn try_from(
        weedle_attributes: &weedle::attribute::ExtendedAttributeList<'_>,
    ) -> Result<Self, Self::Error> {
        let attrs = &weedle_attributes.body.list;

        let mut hash_set = std::collections::HashSet::new();
        for attr in attrs {
            if !hash_set.insert(attr) {
                anyhow::bail!("Duplicated ExtendedAttribute: {:?}", attr);
            }
        }

        attrs
            .iter()
            .map(Attribute::try_from)
            .collect::<Result<Vec<_>, _>>()
            .map(Attributes)
    }
}

/// Represents an "extern C"-style function that will be part of the FFI.
///
/// These can't be declared explicitly in the IDL, but rather, are derived automatically
/// from the high-level interface. Each callable thing in the component API will have a
/// corresponding `FFIFunction` through which it can be invoked, and uniffi also provides
/// some built-in `FFIFunction` helpers for use in the foreign language bindings.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct FFIFunction {
    name: String,
    arguments: Vec<FFIArgument>,
    return_type: Option<FFIType>,
}

impl FFIFunction {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn arguments(&self) -> Vec<&FFIArgument> {
        self.arguments.iter().collect()
    }
    pub fn return_type(&self) -> Option<&FFIType> {
        self.return_type.as_ref()
    }
}

/// Represents an argument to an FFI function.
///
/// Each argument has a name and a type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FFIArgument {
    name: String,
    type_: FFIType,
}

impl FFIArgument {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn type_(&self) -> FFIType {
        self.type_.clone()
    }
}

impl From<&Argument> for FFIArgument {
    fn from(arg: &Argument) -> Self {
        FFIArgument {
            name: arg.name.clone(),
            type_: FFIType::from(&arg.type_),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const IDL1: &'static str = r#"
        namespace foobar{};
        enum Test {
            "test_me",
        };
    "#;

    const IDL2: &'static str = r#"
        namespace hello {
            u64 world();
        };
        dictionary Test {
            boolean me;
        };
    "#;

    #[test]
    fn test_checksum_always_matches_for_same_webidl() {
        for idl in &[IDL1, IDL2] {
            let ci1 = ComponentInterface::from_webidl(idl).unwrap();
            let ci2 = ComponentInterface::from_webidl(idl).unwrap();
            assert_eq!(ci1.checksum(), ci2.checksum());
        }
    }

    #[test]
    fn test_checksum_differs_for_different_webidl() {
        // There is a small probability of this test spuriously failing due to hash collision.
        // If it happens often enough to be a problem, probably this whole "checksum" thing
        // is not working out as intended.
        let ci1 = ComponentInterface::from_webidl(IDL1).unwrap();
        let ci2 = ComponentInterface::from_webidl(IDL2).unwrap();
        assert_ne!(ci1.checksum(), ci2.checksum());
    }

    #[test]
    fn test_checksum_differs_for_different_uniffi_version() {
        // There is a small probability of this test spuriously failing due to hash collision.
        // If it happens often enough to be a problem, probably this whole "checksum" thing
        // is not working out as intended.
        for idl in &[IDL1, IDL2] {
            let ci1 = ComponentInterface::from_webidl(idl).unwrap();
            let mut ci2 = ComponentInterface::from_webidl(idl).unwrap();
            ci2.uniffi_version = String::from("fake-version");
            assert_ne!(ci1.checksum(), ci2.checksum());
        }
    }
}
