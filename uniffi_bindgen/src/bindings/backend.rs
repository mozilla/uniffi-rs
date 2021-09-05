/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! # Backend traits
//!
//! This module provides a number of traits useful for implementing a backend for Uniffi.
//!
//! A `CodeType` is needed for each type that will cross the FFI. It should provide helper machinery
//! in the target language to lift from and lower into a value of that type into a primitive type
//! (the FFIType), and foreign language expressions that call into the machinery. This helper code
//! can be provided by a template file.
//!
//! A `CodeDeclaration` is needed for each type that is declared in the UDL file. This has access to
//! the [ComponentInterface], which is the closest thing to an Intermediate Representation.
//!
//! `CodeDeclaration`s provide the target language's version of the UDL type, including forwarding method calls
//! into Rust. It is likely if you're implementing a `CodeDeclaration` for this purpose, it will also need to cross
//! the FFI, and you'll also need a `CodeType`.
//!
//! `CodeDeclaration`s can also be used to conditionally include code: e.g. only include the CallbackInterfaceRuntime
//! if the user has used at least one callback interface.
//!
//! Each backend has a wrapper template for each file it needs to generate. This should collect the `CodeDeclaration`s that
//! the backend and `ComponentInterface` between them specify and use them to stitch together a file in the target language.
//!
//! The `CodeOracle` provides methods to map the `Type` values found in the `ComponentInterface` to the `CodeType`s specified
//! by the backend. It also provides methods for transforming identifiers into the coding standard for the target language.
//!
//! Each backend will have its own `filter` module, which is used by the askama templates used in all `CodeType`s and `CodeDeclaration`s.
//! This filter provides methods to generate expressions and identifiers in the target language. These are all forwarded to the oracle.

use crate::interface::*;
use crate::Result;
use askama::Template;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashSet;
use std::fmt;
use std::hash::{Hash, Hasher};

pub type TypeIdentifier = Type;
pub type Literal = crate::interface::Literal;

/// An object to look up a foreign language code specific renderer for a given type used.
/// Every `Type` referred to in the `ComponentInterface` should map to a corresponding
/// `CodeType`.
///
/// The mapping may be opaque, but the oracle always knows the answer.
///
/// In adddition, the oracle knows how to render identifiers (function names,
/// class names, variable names etc).
pub trait CodeOracle {
    fn find(&self, type_: &TypeIdentifier) -> Box<dyn CodeType>;

    /// Get the idiomatic rendering of a class name (for enums, records, errors, etc).
    fn class_name(&self, nm: &dyn fmt::Display) -> String;

    /// Get the idiomatic rendering of a function name.
    fn fn_name(&self, nm: &dyn fmt::Display) -> String;

    /// Get the idiomatic rendering of a variable name.
    fn var_name(&self, nm: &dyn fmt::Display) -> String;

    /// Get the idiomatic rendering of an individual enum variant.
    fn enum_variant_name(&self, nm: &dyn fmt::Display) -> String;

    /// Get the idiomatic rendering of an error name.
    fn error_name(&self, nm: &dyn fmt::Display) -> String;

    fn ffi_type_label(&self, ffi_type: &FFIType) -> String;
}

/// A Trait to emit foreign language code to handle referenced types.
/// A type which is specified in the UDL (i.e. a member of the component interface)
/// will have a `CodeDeclaration` as well, but for types used e.g. primitive types, Strings, etc
/// only a `CodeType` is needed.
pub trait CodeType {
    /// The language specific label used to reference this type. This will be used in
    /// method signatures and property declarations.
    fn type_label(&self, oracle: &dyn CodeOracle) -> String;

    /// A representation of this type label that can be used as part of another
    /// identifier. e.g. `read_foo()`, or `FooInternals`.
    ///
    /// This is especially useful when creating specialized objects or methods to deal
    /// with this type only.
    fn canonical_name(&self, oracle: &dyn CodeOracle) -> String {
        self.type_label(oracle)
    }

    /// A representation of the given literal for this type.
    /// N.B. `Literal` is aliased from `interface::Literal`, so may not be whole suited to this task.
    fn literal(&self, oracle: &dyn CodeOracle, _literal: &Literal) -> String {
        unimplemented!("Unimplemented for {}", self.type_label(oracle))
    }

    /// An expression for lowering a value into something we can pass over the FFI.
    ///
    /// N.B. This should align with the `helper_code` generated by this `CodeType`.
    fn lower(&self, oracle: &dyn CodeOracle, _nm: &dyn fmt::Display) -> String {
        unimplemented!("Unimplemented for {}", self.type_label(oracle))
    }

    /// An expression for writing a value into a byte buffer.
    ///
    /// N.B. This should align with the `helper_code` generated by this `CodeType`.
    fn write(
        &self,
        oracle: &dyn CodeOracle,
        _nm: &dyn fmt::Display,
        _target: &dyn fmt::Display,
    ) -> String {
        unimplemented!("Unimplemented for {}", self.type_label(oracle))
    }

    /// An expression for lifting a value from something we received over the FFI.
    ///
    /// N.B. This should align with the `helper_code` generated by this `CodeType`.
    fn lift(&self, oracle: &dyn CodeOracle, _nm: &dyn fmt::Display) -> String {
        unimplemented!("Unimplemented for {}", self.type_label(oracle))
    }

    /// An expression for reading a value from a byte buffer.
    ///
    /// N.B. This should align with the `helper_code` generated by this `CodeType`.
    fn read(&self, oracle: &dyn CodeOracle, _nm: &dyn fmt::Display) -> String {
        unimplemented!("Unimplemented for {}", self.type_label(oracle))
    }

    /// The lift/lower/read/write methods above must be producing expressions that
    /// can be part of a larger statement. Most of the time, that is a function call
    /// to do the work for it.
    /// The functions being called by those experessions should be declared in the
    /// helper code generated by this method.
    fn helper_code(&self, _oracle: &dyn CodeOracle) -> Option<String> {
        None
    }

    /// A list of imports that are needed if this type is in use.
    /// Classes are imported exactly once.
    fn imports(&self, _oracle: &dyn CodeOracle) -> Option<Vec<String>> {
        None
    }
}

/// A trait that is able to render a declaration about a particular member declared in
/// the `ComponentInterface`.
/// Like `CodeType`, it can render declaration code and imports. It also is able to render
/// code at start-up of the FFI.
/// All methods are optional, and there is no requirement that the trait be used for a particular
/// `interface::` member. Thus, it can also be useful for conditionally rendering code.
pub trait CodeDeclaration {
    /// A list of imports that are needed if this type is in use.
    /// Classes are imported exactly once.
    fn imports(&self, _oracle: &dyn CodeOracle) -> Option<Vec<String>> {
        None
    }

    /// Code (one or more statements) that is run on start-up of the library,
    /// but before the client code has access to it.
    fn initialization_code(&self, _oracle: &dyn CodeOracle) -> Option<String> {
        None
    }

    /// Code which represents this member. e.g. the foreign language class definition for
    /// a given Object type.
    fn definition_code(&self, _oracle: &dyn CodeOracle) -> Option<String> {
        None
    }
}

/// Stores a list of rendered templates without duplicates
///
/// This is used for types like `CallbackInterface` and `Object` that need support code that
/// should only be rendered once, even if there are multiples of those types.
#[derive(Default)]
pub struct TemplateRenderSet {
    items: Vec<String>,
    hashes_seen: HashSet<u64>,
}

impl TemplateRenderSet {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn insert<T: 'static + Template + Hash>(&mut self, template: T) -> Result<()> {
        if self.hashes_seen.insert(self.calc_hash(&template)) {
            self.items.push(template.render()?);
        }
        Ok(())
    }

    fn calc_hash<T: 'static + Hash>(&self, template: &T) -> u64 {
        let mut s = DefaultHasher::new();
        // Make sure to include the type id to make things unique
        template.hash(&mut s);
        std::any::TypeId::of::<T>().hash(&mut s);
        s.finish()
    }
}

impl std::iter::IntoIterator for TemplateRenderSet {
    type Item = String;
    type IntoIter = std::vec::IntoIter<String>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}
