/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! This file contains Gecko JS backend-specific types that wrap the UniFFI
//! component interface types. These wrappers are used to generate the WebIDL
//! file and C++ implementation for Firefox.
//!
//! Although UDL (the UniFFI Definition Language) and WebIDL share the same
//! syntax, they have completely different semantics. WebIDL distinguishes
//! between "nullable" (`T?`) and "optional" types (`optional T`), dictionary
//! members are optional by default, dictionaries are not nullable (but can be
//! optional, and must have a default value if they are). The UniFFI type
//! system is much simpler, but can't represent these semantics exactly.
//!
//!
//! The Firefox C++ bindings are also peculiar. A C++ function that implements
//! a WebIDL static or namespace method takes an extra `GlobalObject` argument;
//! methods that return an `ArrayBuffer` also take a `JSContext`; some return
//! values are passed via out parameters while others are returned directly;
//! some WebIDL types map to different C++ types depending on where they're used
//! (in parameters, out parameters, or dictionary members); and throwing
//! functions take an extra `ErrorResult` argument.
//!
//! https://developer.mozilla.org/en-US/docs/Mozilla/WebIDL_bindings describes
//! how Firefox WebIDL constructs are reflected in C++. Since UniFFI is
//! generating both the WebIDL file and the C++ implementation, it must do the
//! same.
//!
//! These combinations of special cases are impossible to express in Askama, so
//! we have a "shadow type system" that wraps the UniFFI interface types.
//! Capturing this logic here lets us keep our templates and filters clean.

use crate::interface::{
    Argument, Constructor, FFIFunction, FFIType, Field, Function, Literal, Method, Object, Record,
    Type,
};

/// WebIDL types correspond to UniFFI interface types, s
#[derive(Clone, Debug)]
pub enum WebIDLType {
    /// Wrapped types include integers, floats, Booleans, strings, and enums.
    /// These wrap the UniFFI type directly, since they have the same names and
    /// semantics.
    Wrapped(Type),

    /// Interfaces correspond to UniFFI objects.
    Interface(String),

    /// Dictionaries correspond to UniFFI records.
    Dictionary(String),

    /// `Nullable` and `Optional` both correspond to UniFFI optional types.
    /// Semantically, "nullable" means "must be passed as an argument or a
    /// dictionary member, but can be `null`". "Optional" means the argument
    /// or member can be omitted entirely.
    Nullable(Box<WebIDLType>),
    Optional(Box<WebIDLType>),

    /// Sequences are the same as their UniFFI counterparts.
    Sequence(Box<WebIDLType>),

    /// Records correspond to UniFFI maps.
    Record(Box<WebIDLType>),
}

impl WebIDLType {
    /// Returns `true` if a WebIDL type is returned via an out parameter in the
    /// C++ implementation; `false` if by value.
    pub fn return_by_out_param(&self) -> bool {
        match self {
            WebIDLType::Wrapped(Type::String) => true,
            WebIDLType::Dictionary(_) | WebIDLType::Record(_) | WebIDLType::Sequence(_) => true,
            WebIDLType::Optional(inner) | WebIDLType::Nullable(inner) => {
                inner.return_by_out_param()
            }
            _ => false,
        }
    }
}

impl From<Type> for WebIDLType {
    fn from(type_: Type) -> WebIDLType {
        match type_ {
            inner @ Type::UInt8
            | inner @ Type::Int8
            | inner @ Type::UInt16
            | inner @ Type::Int16
            | inner @ Type::UInt32
            | inner @ Type::Int32
            | inner @ Type::UInt64
            | inner @ Type::Int64
            | inner @ Type::Float32
            | inner @ Type::Float64
            | inner @ Type::Boolean
            | inner @ Type::String
            | inner @ Type::Enum(_) => WebIDLType::Wrapped(inner),
            Type::Object(name) => WebIDLType::Interface(name),
            Type::Record(name) => WebIDLType::Dictionary(name),
            Type::Error(_) => {
                // TODO: We don't currently throw typed errors; see
                // https://github.com/mozilla/uniffi-rs/issues/295.
                panic!("[TODO: From<Type>({:?})]", type_)
            }
            Type::Optional(inner) => match *inner {
                Type::Record(name) => WebIDLType::Optional(WebIDLType::Dictionary(name).into()),
                inner => WebIDLType::Nullable(Box::new(inner.into())),
            },
            Type::Sequence(inner) => WebIDLType::Sequence(Box::new((*inner).into())),
            Type::Map(inner) => WebIDLType::Record(Box::new((*inner).into())),
        }
    }
}

impl From<&WebIDLType> for FFIType {
    fn from(type_: &WebIDLType) -> FFIType {
        match type_ {
            WebIDLType::Wrapped(inner) => inner.into(),
            WebIDLType::Interface(_) => FFIType::UInt64,
            WebIDLType::Dictionary(_)
            | WebIDLType::Optional(_)
            | WebIDLType::Nullable(_)
            | WebIDLType::Sequence(_)
            | WebIDLType::Record(_) => FFIType::RustBuffer,
        }
    }
}

/// A WebIDL interface reflects a UniFFI object.
pub struct WebIDLInterface(Object);

impl WebIDLInterface {
    /// Wraps the given UniFFI object.
    pub fn new(obj: Object) -> Self {
        WebIDLInterface(obj)
    }

    /// Returns the name of this interface.
    pub fn name(&self) -> &str {
        self.0.name()
    }

    /// Returns the FFI function used to deallocate an instance of this
    /// interface.
    pub fn ffi_object_free(&self) -> &FFIFunction {
        self.0.ffi_object_free()
    }

    /// Returns a list of wrapped constructors for this interface.
    pub fn constructors(&self) -> Vec<WebIDLConstructor<'_>> {
        self.0
            .constructors()
            .into_iter()
            .map(WebIDLConstructor::new)
            .collect()
    }

    /// Returns a list of wrapped methods for this interface.
    pub fn methods(&self) -> Vec<WebIDLMethod<'_>> {
        self.0
            .methods()
            .into_iter()
            .map(WebIDLMethod::new)
            .collect()
    }
}

/// A WebIDL function reflects a UniFFI top-level function.
pub struct WebIDLFunction(Function);

impl WebIDLFunction {
    /// Wraps the given UniFFI object.
    pub fn new(func: Function) -> Self {
        WebIDLFunction(func)
    }

    /// Returns the name of this function.
    pub fn name(&self) -> &str {
        self.0.name()
    }

    /// Returns the FFI function that corresponds to this function.
    pub fn ffi_func(&self) -> &FFIFunction {
        self.0.ffi_func()
    }

    /// Returns a list of arguments to declare for this function in the WebIDL
    /// declaration.
    pub fn webidl_arguments(&self) -> Vec<WebIDLArgument<'_>> {
        self.0
            .arguments()
            .into_iter()
            .map(|arg| WebIDLArgument(arg))
            .collect()
    }

    /// Returns the WebIDL return type of this function.
    pub fn webidl_return_type(&self) -> Option<WebIDLType> {
        self.0.return_type().cloned().map(WebIDLType::from)
    }

    /// Indicates whether this function can throw an error.
    pub fn throws(&self) -> bool {
        self.0.throws().is_some()
    }

    /// Returns a list of arguments to declare for this function in the C++
    /// implementation, including any extras and out parameters.
    pub fn cpp_arguments(&self) -> Vec<BindingArgument<'_>> {
        let args = self.webidl_arguments();
        let mut result = Vec::with_capacity(args.len() + 3);
        // All static methods take a `GlobalObject`.
        result.push(BindingArgument::GlobalObject);
        // ...Then the declared WebIDL arguments...
        result.extend(args.into_iter().map(|arg| BindingArgument::In(arg)));
        // ...Then the out param, depending on the return type.
        if let Some(type_) = self
            .webidl_return_type()
            .filter(|type_| type_.return_by_out_param())
        {
            result.push(BindingArgument::Out(type_));
        }
        // ...And finally, the `ErrorResult` to throw errors.
        if self.throws() {
            result.push(BindingArgument::ErrorResult);
        }
        result
    }

    /// Returns the C++ return type of this function, or `None` if the function
    /// doesn't return a value, or returns it via an out parameter.
    pub fn cpp_return_type(&self) -> Option<WebIDLType> {
        self.webidl_return_type()
            .filter(|type_| !type_.return_by_out_param())
    }

    /// Indicates how this function returns its result, either by value or via
    /// an out parameter.
    pub fn cpp_return_by(&self) -> ReturnBy {
        self.webidl_return_type()
            .map(|v| ReturnBy::from_return_type(&v))
            .unwrap_or(ReturnBy::Void)
    }

    /// Indicates how this function throws errors, either by an `ErrorResult`
    /// parameter, or by a fatal assertion.
    pub fn cpp_throw_by(&self) -> ThrowBy {
        if self.throws() {
            ThrowBy::ErrorResult("aRv")
        } else {
            ThrowBy::Assert
        }
    }
}

/// A WebIDL constructor wraps a UniFFI object constructor.
pub struct WebIDLConstructor<'a>(&'a Constructor);

impl<'a> WebIDLConstructor<'a> {
    /// Wraps the given UniFFI object constructor.
    pub fn new(cons: &'a Constructor) -> Self {
        WebIDLConstructor(cons)
    }

    /// Returns the name of this constructor.
    pub fn name(&self) -> &'a str {
        self.0.name()
    }

    /// Returns the FFI function that corresponds to this constructor.
    pub fn ffi_func(&self) -> &'a FFIFunction {
        self.0.ffi_func()
    }

    /// Returns a list of arguments to declare for this constructor in the
    /// WebIDL declaration.
    pub fn webidl_arguments(&self) -> Vec<WebIDLArgument<'_>> {
        self.0
            .arguments()
            .into_iter()
            .map(|arg| WebIDLArgument(arg))
            .collect()
    }

    /// Indicates whether this constructor can throw an error.
    pub fn throws(&self) -> bool {
        self.0.throws().is_some()
    }

    /// Returns a list of arguments to declare for this constructor in the C++
    /// implementation, including any extras and out parameters.
    pub fn cpp_arguments(&self) -> Vec<BindingArgument<'_>> {
        let args = self.webidl_arguments();
        let mut result = Vec::with_capacity(args.len() + 2);
        // First the `GlobalObject`, just like for static methods...
        result.push(BindingArgument::GlobalObject);
        result.extend(args.into_iter().map(|arg| BindingArgument::In(arg)));
        // Constructors never take out params, since they must return an
        // instance of the object.
        if self.throws() {
            // ...But they can throw, so pass an `ErrorResult` if we need to
            // throw errors.
            result.push(BindingArgument::ErrorResult);
        }
        result
    }

    /// Indicates how this constructor throws errors, either by an `ErrorResult`
    /// parameter, or by a fatal assertion.
    pub fn cpp_throw_by(&self) -> ThrowBy {
        if self.throws() {
            ThrowBy::ErrorResult("aRv")
        } else {
            ThrowBy::Assert
        }
    }
}

/// A WebIDL instance method wraps a UniFFI object method.
pub struct WebIDLMethod<'a>(&'a Method);

impl<'a> WebIDLMethod<'a> {
    /// Wraps the given UniFFI object method.
    pub fn new(method: &'a Method) -> Self {
        WebIDLMethod(method)
    }

    /// Returns the name of this method.
    pub fn name(&self) -> &'a str {
        self.0.name()
    }

    /// Returns the FFI function that corresponds to this method.
    pub fn ffi_func(&self) -> &'a FFIFunction {
        self.0.ffi_func()
    }

    /// Returns a list of arguments to declare for this method in the
    /// WebIDL declaration.
    pub fn webidl_arguments(&self) -> Vec<WebIDLArgument<'_>> {
        self.0
            .arguments()
            .into_iter()
            .map(|arg| WebIDLArgument(arg))
            .collect()
    }

    /// Returns the WebIDL return type of this method.
    pub fn webidl_return_type(&self) -> Option<WebIDLType> {
        self.0.return_type().cloned().map(WebIDLType::from)
    }

    /// Indicates whether this method can throw an error.
    pub fn throws(&self) -> bool {
        self.0.throws().is_some()
    }

    /// Returns a list of arguments to declare for this method in the C++
    /// implementation, including any extras and out parameters.
    pub fn cpp_arguments(&self) -> Vec<BindingArgument<'_>> {
        let args = self.webidl_arguments();
        let mut result = Vec::with_capacity(args.len() + 2);
        // Methods don't take a `GlobalObject` as their first argument.
        result.extend(args.into_iter().map(|arg| BindingArgument::In(arg)));
        if let Some(type_) = self
            .webidl_return_type()
            .filter(|type_| type_.return_by_out_param())
        {
            // ...But they can take out params, since they return values.
            result.push(BindingArgument::Out(type_));
        }
        if self.throws() {
            // ...And they can throw.
            result.push(BindingArgument::ErrorResult);
        }
        result
    }

    /// Returns the C++ return type of this function, or `None` if the method
    /// doesn't return a value, or returns it via an out parameter.
    pub fn cpp_return_type(&self) -> Option<WebIDLType> {
        self.webidl_return_type()
            .filter(|type_| !type_.return_by_out_param())
    }

    /// Indicates how this function returns its result, either by value or via
    /// an out parameter.
    pub fn cpp_return_by(&self) -> ReturnBy {
        self.webidl_return_type()
            .map(|v| ReturnBy::from_return_type(&v))
            .unwrap_or(ReturnBy::Void)
    }

    /// Indicates how this method throws errors, either by an `ErrorResult`
    /// parameter, or by a fatal assertion.
    pub fn cpp_throw_by(&self) -> ThrowBy {
        if self.throws() {
            ThrowBy::ErrorResult("aRv")
        } else {
            ThrowBy::Assert
        }
    }
}

/// A WebIDL dictionary method wraps a UniFFI record.
pub struct WebIDLDictionary(Record);

impl WebIDLDictionary {
    /// Wraps the given UniFFI record.
    pub fn new(record: Record) -> Self {
        WebIDLDictionary(record)
    }

    /// Returns the name of this dictionary.
    pub fn name(&self) -> &str {
        self.0.name()
    }

    /// Returns the members of this dictionary.
    pub fn members(&self) -> Vec<WebIDLMember<'_>> {
        self.0.fields().into_iter().map(WebIDLMember).collect()
    }
}

/// A WebIDL argument wraps a UniFFI function, or constructor argument.
pub struct WebIDLArgument<'a>(&'a Argument);

impl<'a> WebIDLArgument<'a> {
    /// Returns the argument name.
    pub fn name(&self) -> &'a str {
        self.0.name()
    }

    /// Returns the argument type.
    pub fn type_(&self) -> WebIDLType {
        self.0.type_().into()
    }

    /// Indicates if the argument should have an `optional` keyword. `true`
    /// for nullable dictionaries and arguments that declare a default value
    /// in UDL; `false` otherwise.
    pub fn optional(&self) -> bool {
        match self.0.type_() {
            Type::Optional(inner) => matches!(inner.as_ref(), Type::Record(_)),
            _ => self.0.default_value().is_some(),
        }
    }

    /// Returns the default value for this argument, if one is specified.
    pub fn default_value(&self) -> Option<Literal> {
        if let Some(literal) = self.0.default_value() {
            return Some(literal);
        }
        match self.0.type_() {
            Type::Optional(inner) => match inner.as_ref() {
                // Nullable UDL dictionaries must declare a default value
                // in WebIDL.
                Type::Record(_) => Some(Literal::EmptyMap),
                _ => None,
            },
            _ => None,
        }
    }
}

/// A WebIDL dictionary member wraps a UniFFI record field.
pub struct WebIDLMember<'a>(&'a Field);

impl<'a> WebIDLMember<'a> {
    /// Returns the member name.
    pub fn name(&self) -> &'a str {
        self.0.name()
    }

    /// Returns the member type.
    pub fn type_(&self) -> WebIDLType {
        match self.0.type_() {
            Type::Optional(inner) => WebIDLType::Optional(Box::new((*inner).into())),
            type_ => type_.into(),
        }
        /*match self.0.type_() {
            Type::Optional(inner) => match *inner {
                // As with arguments, nullable dictionaries are illegal in
                // WebIDL.
                Type::Record(name) => Type::Record(name),
                inner => Type::Optional(inner.into()),
            },
            type_ => type_
        }*/
    }

    /// Indicates if the member should have a `required` keyword. In UDL, all
    /// dictionary members are required by default; in WebIDL, they're optional.
    /// For WebIDL, all members are `required`, except for nullable
    /// dictionaries, which must be optional.
    pub fn required(&self) -> bool {
        match self.0.type_() {
            Type::Optional(_) => false,
            _ => true,
        }
    }

    /// Returns the default value for this member, if one is defined.
    pub fn default_value(&self) -> Option<Literal> {
        match self.0.type_() {
            Type::Optional(inner) => match inner.as_ref() {
                // Nullable UDL dictionaries must declare a default value
                // in WebIDL.
                Type::Record(_) => Some(Literal::EmptyMap),
                _ => None,
            },
            _ => None,
        }
    }
}

/// Describes how a function returns its result.
pub enum ReturnBy {
    /// The function returns its result in an out parameter with the given
    /// name and type.
    OutParam(&'static str, WebIDLType),

    /// The function returns its result by value.
    Value(WebIDLType),

    /// The function doesn't declare a return type.
    Void,
}

impl ReturnBy {
    fn from_return_type(type_: &WebIDLType) -> Self {
        if type_.return_by_out_param() {
            ReturnBy::OutParam("aRetVal", type_.clone())
        } else {
            ReturnBy::Value(type_.clone())
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
    In(WebIDLArgument<'a>),

    /// The argument is an out parameter used to return results by reference.
    Out(WebIDLType),
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
