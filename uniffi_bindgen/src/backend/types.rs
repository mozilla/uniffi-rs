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
//! A `CodeDeclaration` is needed for each type that is declared in the UDL file. This has access to
//! the [crate::interface::ComponentInterface], which is the closest thing to an Intermediate Representation.
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

use super::{CodeOracle, Literal};
use crate::interface::*;

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

    /// Name of the FfiConverter
    ///
    /// This is the object that contains the lower, write, lift, and read methods for this type.
    /// Depending on the binding this will either be a singleton or a class with static methods.
    ///
    /// This is the newer way of handling these methods and replaces the lower, write, lift, and
    /// read CodeType methods.  Currently only used by Kotlin, but the plan is to move other
    /// backends to using this.
    fn ffi_converter_name(&self, oracle: &dyn CodeOracle) -> String {
        oracle.class_name(&format!("FfiConverter{}", self.canonical_name(oracle)))
    }

    /// An expression for lowering a value into something we can pass over the FFI.
    fn lower(&self, oracle: &dyn CodeOracle) -> String {
        format!("{}.lower", self.ffi_converter_name(oracle))
    }

    /// An expression for writing a value into a byte buffer.
    fn write(&self, oracle: &dyn CodeOracle) -> String {
        format!("{}.write", self.ffi_converter_name(oracle))
    }

    /// An expression for lifting a value from something we received over the FFI.
    fn lift(&self, oracle: &dyn CodeOracle) -> String {
        format!("{}.lift", self.ffi_converter_name(oracle))
    }

    /// An expression for reading a value from a byte buffer.
    fn read(&self, oracle: &dyn CodeOracle) -> String {
        format!("{}.read", self.ffi_converter_name(oracle))
    }

    /// A list of imports that are needed if this type is in use.
    /// Classes are imported exactly once.
    fn imports(&self, _oracle: &dyn CodeOracle) -> Option<Vec<String>> {
        None
    }

    /// Function to run at startup
    fn initialization_fn(&self, _oracle: &dyn CodeOracle) -> Option<String> {
        None
    }

    /// An expression to coerce the given variable to the expected type.
    fn coerce(&self, oracle: &dyn CodeOracle, _nm: &str) -> String {
        panic!("Unimplemented for {}", self.type_label(oracle));
    }
}

/// This trait is used to implement `CodeType` for `Type` and type-like structs (`Record`, `Enum`, `Field`,
/// etc).  We forward all method calls to a `Box<dyn CodeType>`, which we get by calling
/// `CodeOracle.find()`.
pub trait CodeTypeDispatch {
    fn code_type_impl(&self, oracle: &dyn CodeOracle) -> Box<dyn CodeType>;
}

impl CodeTypeDispatch for Type {
    fn code_type_impl(&self, oracle: &dyn CodeOracle) -> Box<dyn CodeType> {
        oracle.find(self)
    }
}

impl CodeTypeDispatch for Record {
    fn code_type_impl(&self, oracle: &dyn CodeOracle) -> Box<dyn CodeType> {
        oracle.find(&self.type_())
    }
}

impl CodeTypeDispatch for Enum {
    fn code_type_impl(&self, oracle: &dyn CodeOracle) -> Box<dyn CodeType> {
        oracle.find(&self.type_())
    }
}

impl CodeTypeDispatch for Error {
    fn code_type_impl(&self, oracle: &dyn CodeOracle) -> Box<dyn CodeType> {
        oracle.find(&self.type_())
    }
}

impl CodeTypeDispatch for Object {
    fn code_type_impl(&self, oracle: &dyn CodeOracle) -> Box<dyn CodeType> {
        oracle.find(&self.type_())
    }
}

impl CodeTypeDispatch for CallbackInterface {
    fn code_type_impl(&self, oracle: &dyn CodeOracle) -> Box<dyn CodeType> {
        oracle.find(&self.type_())
    }
}

impl CodeTypeDispatch for Field {
    fn code_type_impl(&self, oracle: &dyn CodeOracle) -> Box<dyn CodeType> {
        oracle.find(self.type_())
    }
}

impl CodeTypeDispatch for Argument {
    fn code_type_impl(&self, oracle: &dyn CodeOracle) -> Box<dyn CodeType> {
        oracle.find(self.type_())
    }
}

// Needed to handle &&Type and &&&Type values, which we sometimes end up with in the template code
impl<T, C> CodeTypeDispatch for T
where
    T: std::ops::Deref<Target = C>,
    C: CodeTypeDispatch,
{
    fn code_type_impl(&self, oracle: &dyn CodeOracle) -> Box<dyn CodeType> {
        self.deref().code_type_impl(oracle)
    }
}

impl<T: CodeTypeDispatch> CodeType for T {
    // The above code implements `CodeTypeDispatch` for `Type` and type-like structs (`Record`,
    // `Enum`, `Field`, etc).  Now we can leverage that to implement `CodeType` for all of them.
    // This allows for simpler template code (`field|lower` instead of `field.type_()|lower`)
    fn type_label(&self, oracle: &dyn CodeOracle) -> String {
        self.code_type_impl(oracle).type_label(oracle)
    }

    fn canonical_name(&self, oracle: &dyn CodeOracle) -> String {
        self.code_type_impl(oracle).canonical_name(oracle)
    }

    fn literal(&self, oracle: &dyn CodeOracle, literal: &Literal) -> String {
        self.code_type_impl(oracle).literal(oracle, literal)
    }

    fn lower(&self, oracle: &dyn CodeOracle) -> String {
        self.code_type_impl(oracle).lower(oracle)
    }

    fn write(&self, oracle: &dyn CodeOracle) -> String {
        self.code_type_impl(oracle).write(oracle)
    }

    fn lift(&self, oracle: &dyn CodeOracle) -> String {
        self.code_type_impl(oracle).lift(oracle)
    }

    fn read(&self, oracle: &dyn CodeOracle) -> String {
        self.code_type_impl(oracle).read(oracle)
    }

    fn imports(&self, oracle: &dyn CodeOracle) -> Option<Vec<String>> {
        self.code_type_impl(oracle).imports(oracle)
    }

    fn initialization_fn(&self, oracle: &dyn CodeOracle) -> Option<String> {
        self.code_type_impl(oracle).initialization_fn(oracle)
    }

    fn coerce(&self, oracle: &dyn CodeOracle, nm: &str) -> String {
        self.code_type_impl(oracle).coerce(oracle, nm)
    }
}
