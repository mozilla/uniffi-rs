/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! # Backend traits
//!
//! This module provides a number of traits useful for implementing a backend for Uniffi.
//!
//! A `CodeType` is needed for each type that will cross the FFI. It should provide helper machinery
//! in the target language to lift from and lower into a value of that type into a primitive type
//! (the FfiType), and foreign language expressions that call into the machinery. This helper code
//! can be provided by a template file.
//!
//! The `CodeOracle` provides methods to map the `Type` values found in the `ComponentInterface` to the `CodeType`s specified
//! by the backend. It also provides methods for transforming identifiers into the coding standard for the target language.
//!
//! There's also a CodeTypeDispatch trait, implemented for every type, which allows a CodeType to be created
//! by the specified `CodeOracle`. This means backends are able to provide a custom CodeType for each type
//! via that backend's CodeOracle.
//!
//! Each backend will have its own `filter` module, which is used by the askama templates used in all `CodeType`s and `CodeDeclaration`s.
//! This filter provides methods to generate expressions and identifiers in the target language. These are all forwarded to the oracle.

use std::fmt::Debug;

use crate::interface::*;

/// A Trait to help render types in a language specific format.
pub trait CodeType: Debug {
    /// The language specific label used to reference this type. This will be used in
    /// method signatures and property declarations.
    fn type_label(&self) -> String;

    /// A representation of this type label that can be used as part of another
    /// identifier. e.g. `read_foo()`, or `FooInternals`.
    ///
    /// This is especially useful when creating specialized objects or methods to deal
    /// with this type only.
    fn canonical_name(&self) -> String {
        self.type_label()
    }

    /// A representation of the given literal for this type.
    /// N.B. `Literal` is aliased from `interface::Literal`, so may not be whole suited to this task.
    fn literal(&self, _literal: &Literal) -> String {
        unimplemented!("Unimplemented for {}", self.type_label())
    }

    /// Name of the FfiConverter
    ///
    /// This is the object that contains the lower, write, lift, and read methods for this type.
    /// Depending on the binding this will either be a singleton or a class with static methods.
    ///
    /// This is the newer way of handling these methods and replaces the lower, write, lift, and
    /// read CodeType methods.  Currently only used by Kotlin, but the plan is to move other
    /// backends to using this.
    fn ffi_converter_name(&self) -> String {
        format!("FfiConverter{}", self.canonical_name())
    }

    /// An expression for lowering a value into something we can pass over the FFI.
    fn lower(&self) -> String {
        format!("{}.lower", self.ffi_converter_name())
    }

    /// An expression for writing a value into a byte buffer.
    fn write(&self) -> String {
        format!("{}.write", self.ffi_converter_name())
    }

    /// An expression for lifting a value from something we received over the FFI.
    fn lift(&self) -> String {
        format!("{}.lift", self.ffi_converter_name())
    }

    /// An expression for reading a value from a byte buffer.
    fn read(&self) -> String {
        format!("{}.read", self.ffi_converter_name())
    }

    /// A list of imports that are needed if this type is in use.
    /// Classes are imported exactly once.
    fn imports(&self) -> Option<Vec<String>> {
        None
    }

    /// Function to run at startup
    fn initialization_fn(&self) -> Option<String> {
        None
    }
}
