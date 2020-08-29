/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Helpers for generating C++ WebIDL bindings from a UniFFI component
//! interface.
//!
//! The C++ bindings that we generate for Firefox are a little peculiar.
//! Depending on how they're declared in WebIDL, some methods take extra
//! arguments. For example, static methods take a `GlobalObject`, methods that
//! return an `ArrayBuffer` also take a `JSContext`, some return values are
//! passed via out parameters while others are returned directly, some
//! arguments map to different C++ types depending on whether they're in or out
//! parameters, and throwing functions also take an `ErrorResult`.
//!
//! These conditions and combinations are tricky to express in Askama, so we
//! handle them in extension traits on the UniFFI types, and keep our templates
//! clean.

use crate::interface::{Argument, Constructor, Function, Method, Type};

/// Extension methods for all functions: top-level functions, constructors, and
/// methods.
pub trait BindingFunction {
    /// Returns a list of arguments to declare for this function, including any
    /// extras and out parameters.
    fn binding_arguments(&self) -> Vec<BindingArgument<'_>>;

    /// Indicates how errors should be thrown from this function, either by an
    /// `ErrorResult` parameter, or by a fatal assertion.
    fn throw_by(&self) -> ThrowBy;
}

/// Extension methods for functions that return a value of any type. Excludes
/// constructors, which must return an instance of their type.
pub trait ReturningBindingFunction: BindingFunction {
    /// Returns the return type for this function, or `None` if the function
    /// doesn't return a value, or returns it via an out parameter.
    fn binding_return_type(&self) -> Option<&Type>;

    /// Indicates how results should be returned, either by value or via an out
    /// parameter.
    fn return_by(&self) -> ReturnBy<'_>;
}

impl BindingFunction for Function {
    fn binding_arguments(&self) -> Vec<BindingArgument<'_>> {
        let args = self.arguments();
        let mut result = Vec::with_capacity(args.len() + 3);
        result.push(BindingArgument::GlobalObject);
        result.extend(args.into_iter().map(|arg| BindingArgument::In(arg)));
        if let Some(type_) = self.return_type().filter(|type_| is_out_param_type(type_)) {
            result.push(BindingArgument::Out(type_));
        }
        if self.throws().is_some() {
            result.push(BindingArgument::ErrorResult);
        }
        result
    }

    fn throw_by(&self) -> ThrowBy {
        if self.throws().is_some() {
            ThrowBy::ErrorResult("aRv")
        } else {
            ThrowBy::Assert
        }
    }
}

impl ReturningBindingFunction for Function {
    fn binding_return_type(&self) -> Option<&Type> {
        self.return_type().filter(|type_| !is_out_param_type(type_))
    }

    fn return_by(&self) -> ReturnBy<'_> {
        self.return_type()
            .map(ReturnBy::from_return_type)
            .unwrap_or(ReturnBy::Void)
    }
}

impl BindingFunction for Constructor {
    fn binding_arguments(&self) -> Vec<BindingArgument<'_>> {
        let args = self.arguments();
        let mut result = Vec::with_capacity(args.len() + 2);
        result.push(BindingArgument::GlobalObject);
        result.extend(args.into_iter().map(|arg| BindingArgument::In(arg)));
        // Constructors never take out params, since they must return an
        // instance of the object.
        if self.throws().is_some() {
            result.push(BindingArgument::ErrorResult);
        }
        result
    }

    fn throw_by(&self) -> ThrowBy {
        if self.throws().is_some() {
            ThrowBy::ErrorResult("aRv")
        } else {
            ThrowBy::Assert
        }
    }
}

impl BindingFunction for Method {
    fn binding_arguments(&self) -> Vec<BindingArgument<'_>> {
        let args = self.arguments();
        let mut result = Vec::with_capacity(args.len() + 2);
        // Methods don't take a `GlobalObject` as their first argument.
        result.extend(args.into_iter().map(|arg| BindingArgument::In(arg)));
        if let Some(type_) = self.return_type().filter(|type_| is_out_param_type(type_)) {
            result.push(BindingArgument::Out(type_));
        }
        if self.throws().is_some() {
            result.push(BindingArgument::ErrorResult);
        }
        result
    }

    fn throw_by(&self) -> ThrowBy {
        if self.throws().is_some() {
            ThrowBy::ErrorResult("aRv")
        } else {
            ThrowBy::Assert
        }
    }
}

impl ReturningBindingFunction for Method {
    fn binding_return_type(&self) -> Option<&Type> {
        self.return_type().filter(|type_| !is_out_param_type(type_))
    }

    fn return_by(&self) -> ReturnBy<'_> {
        self.return_type()
            .map(ReturnBy::from_return_type)
            .unwrap_or(ReturnBy::Void)
    }
}

/// Returns `true` if a type is returned via an out parameter; `false` if
/// by value.
fn is_out_param_type(type_: &Type) -> bool {
    matches!(type_, Type::String | Type::Optional(_) | Type::Record(_) | Type::Map(_) | Type::Sequence(_))
}

/// Describes how a function returns its result.
pub enum ReturnBy<'a> {
    /// The function returns its result in an out parameter with the given
    /// name and type.
    OutParam(&'static str, &'a Type),

    /// The function returns its result by value.
    Value(&'a Type),

    /// The function doesn't declare a return type.
    Void,
}

impl<'a> ReturnBy<'a> {
    fn from_return_type(type_: &'a Type) -> Self {
        if is_out_param_type(type_) {
            ReturnBy::OutParam("aRetVal", type_)
        } else {
            ReturnBy::Value(type_)
        }
    }
}

/// Describes how a function throws errors.
pub enum ThrowBy {
    /// Errors should be set on the `ErrorResult` parameter with the given
    /// name.
    ErrorResult(&'static str),

    /// Errors should fatally assert.
    Assert,
}

/// Describes a function argument.
pub enum BindingArgument<'a> {
    /// The argument is a `GlobalObject`, passed to constructors, static, and
    /// namespace methods.
    GlobalObject,

    /// The argument is an `ErrorResult`, passed to throwing functions.
    ErrorResult,

    /// The argument is a normal input parameter.
    In(&'a Argument),

    /// The argument is an out parameter used to return results by reference.
    Out(&'a Type),
}

impl<'a> BindingArgument<'a> {
    /// Returns the argument name.
    pub fn name(&self) -> &'a str {
        match self {
            BindingArgument::GlobalObject => "aGlobal",
            BindingArgument::ErrorResult => "aRv",
            BindingArgument::In(arg) => arg.name(),
            BindingArgument::Out(_) => "aRetVal",
        }
    }
}
