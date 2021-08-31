/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::interface::*;
use std::fmt;

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

    /// Get the idiomatic rendering of a module
    fn mod_name(&self, nm: &dyn fmt::Display) -> String;

    /// Get the idiomatic rendering of an exception name
    ///
    /// This replaces "Error" at the end of the name with "Exception".  Rust code typically uses
    /// "Error" for any type of error but in the Java world, "Error" means a non-recoverable error
    /// and is distinguished from an "Exception".
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
    fn literal(&self, oracle: &dyn CodeOracle, literal: &Literal) -> String;

    /// An expression for lowering a value into something we can pass over the FFI.
    ///
    /// N.B. This should align with the `helper_code` generated by this `CodeType`.
    fn lower(&self, _oracle: &dyn CodeOracle, _nm: &dyn fmt::Display) -> String {
        panic!("lower not implemented")
    }

    /// An expression for writing a value into a byte buffer.
    ///
    /// N.B. This should align with the `helper_code` generated by this `CodeType`.
    fn write(
        &self,
        _oracle: &dyn CodeOracle,
        _nm: &dyn fmt::Display,
        _target: &dyn fmt::Display,
    ) -> String {
        panic!("write not implemented")
    }

    /// An expression for lifting a value from something we received over the FFI.
    ///
    /// N.B. This should align with the `helper_code` generated by this `CodeType`.
    fn lift(&self, _oracle: &dyn CodeOracle, _nm: &dyn fmt::Display) -> String {
        panic!("lift not implemented")
    }

    /// An expression for reading a value from a byte buffer.
    ///
    /// N.B. This should align with the `helper_code` generated by this `CodeType`.
    fn read(&self, _oracle: &dyn CodeOracle, _nm: &dyn fmt::Display) -> String {
        panic!("read not implemented")
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
    fn import_code(&self, _oracle: &dyn CodeOracle) -> Option<Vec<String>> {
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
    fn import_code(&self, _oracle: &dyn CodeOracle) -> Option<Vec<String>> {
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
