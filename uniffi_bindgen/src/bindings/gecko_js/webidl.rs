/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! This file contains Gecko JS backend-specific extensions for the UniFFI
//! component interface types. These extensions are used to generate the WebIDL
//! file and C++ implementation for Firefox.
//!
//! Although UDL (the UniFFI Definition Language) and WebIDL share the same
//! syntax, they have completely different semantics. WebIDL distinguishes
//! between "nullable" (`T?`) and "optional" types (`optional T`), dictionary
//! members are optional by default, dictionaries are not nullable (but can be
//! optional, and must have a default value if they are). The UniFFI type
//! system is much simpler, but can't represent these semantics exactly.
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
//! we have a "shadow type system" with extension traits implemented for the
//! UniFFI interface types. Capturing this logic here lets us keep our templates
//! and filters clean.

use crate::interface::{Argument, Constructor, FFIType, Field, Function, Literal, Method, Type};

/// WebIDL types correspond to UniFFI interface types, but carry additional
/// information for compound types.
#[derive(Debug)]
pub enum WebIDLType<'a> {
    /// Flat (non-recursive) types include integers, floats, Booleans, strings,
    /// enums, objects (called "interfaces" in WebIDL), and records
    /// ("dictionaries"). These don't have special semantics, so we just wrap
    /// the underlying UniFFI type.
    Flat(&'a Type),

    /// `Nullable` and `Optional` both correspond to UniFFI optional types.
    /// Semantically, "nullable" means "must be passed as an argument or a
    /// dictionary member, but can be `null`". "Optional" means the argument
    /// or member can be omitted entirely, or set to `undefined`.
    Nullable(Box<WebIDLType<'a>>),
    Optional(Box<WebIDLType<'a>>),

    /// Optionals with a default value are a grab bag of special cases in Gecko.
    /// In the generated C++ bindings, the type of an optional with a default
    /// value is `T`, not `Optional<T>`. However, it must be serialized as if
    /// it's an `Optional<T>`, since that's what the Rust side of the FFI
    /// expects.
    OptionalWithDefaultValue(Box<WebIDLType<'a>>),

    /// Sequences are the same as their UniFFI counterparts.
    Sequence(Box<WebIDLType<'a>>),

    /// Maps correspond to WebIDL records.
    Map(Box<WebIDLType<'a>>),
}

impl<'a> WebIDLType<'a> {
    /// Returns `true` if the WebIDL type is returned via an out parameter in
    /// the C++ implementation; `false` if by value.
    pub fn needs_out_param(&self) -> bool {
        match self {
            WebIDLType::Flat(Type::String) | WebIDLType::Flat(Type::Record(_)) => true,
            WebIDLType::Map(_) | WebIDLType::Sequence(_) => true,
            WebIDLType::Optional(inner)
            | WebIDLType::OptionalWithDefaultValue(inner)
            | WebIDLType::Nullable(inner) => inner.needs_out_param(),
            _ => false,
        }
    }

    pub fn is_optional_record(&self) -> bool {
        match self {
            WebIDLType::OptionalWithDefaultValue(inner) => {
                matches!(inner.as_ref(), WebIDLType::Flat(Type::Record(_)))
            }
            _ => false,
        }
    }
}

impl<'a> From<&'a Type> for WebIDLType<'a> {
    fn from(type_: &'a Type) -> WebIDLType<'a> {
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
            | inner @ Type::Enum(_)
            | inner @ Type::Object(_)
            | inner @ Type::Record(_) => WebIDLType::Flat(inner),
            Type::Error(_) => {
                // TODO: We don't currently throw typed errors; see
                // https://github.com/mozilla/uniffi-rs/issues/295.
                panic!("[TODO: From<Type>({:?})]", type_)
            }
            Type::Timestamp => panic!("Timestamp unimplemented"),
            Type::Duration => panic!("Duration unimplemented"),
            Type::CallbackInterface(_) => panic!("Callback interfaces unimplemented"),
            Type::Optional(inner) => match &**inner {
                Type::Record(_) => {
                    WebIDLType::OptionalWithDefaultValue(Box::new((&**inner).into()))
                }
                inner => WebIDLType::Nullable(Box::new(inner.into())),
            },
            Type::Sequence(inner) => WebIDLType::Sequence(Box::new((&**inner).into())),
            Type::Map(inner) => WebIDLType::Map(Box::new((&**inner).into())),
        }
    }
}

impl<'a> From<&WebIDLType<'a>> for FFIType {
    fn from(type_: &WebIDLType<'a>) -> FFIType {
        match type_ {
            WebIDLType::Flat(inner) => (*inner).into(),
            WebIDLType::Optional(_)
            | WebIDLType::OptionalWithDefaultValue(_)
            | WebIDLType::Nullable(_)
            | WebIDLType::Sequence(_)
            | WebIDLType::Map(_) => FFIType::RustBuffer,
        }
    }
}

/// Extensions to support WebIDL namespace methods.
pub trait FunctionExt {
    /// Returns the WebIDL return type of this function.
    fn webidl_return_type(&self) -> Option<WebIDLType<'_>>;

    /// Returns a list of arguments to declare for this function in the C++
    /// implementation, including any extras and out parameters.
    fn cpp_arguments(&self) -> Vec<CPPArgument<'_>>;

    /// Returns the C++ return type of this function, or `None` if the function
    /// doesn't return a value, or returns it via an out parameter.
    fn cpp_return_type(&self) -> Option<WebIDLType<'_>>;

    /// Indicates how this function returns its result, either by value or via
    /// an out parameter.
    fn cpp_return_by(&self) -> ReturnBy<'_>;

    /// Indicates how this function throws errors, either by an `ErrorResult`
    /// parameter, or by a fatal assertion.
    fn cpp_throw_by(&self) -> ThrowBy;
}

impl FunctionExt for Function {
    fn webidl_return_type(&self) -> Option<WebIDLType<'_>> {
        self.return_type().map(WebIDLType::from)
    }

    fn cpp_arguments(&self) -> Vec<CPPArgument<'_>> {
        let args = self.arguments();
        let mut result = Vec::with_capacity(args.len() + 3);
        // All static methods take a `GlobalObject`.
        result.push(CPPArgument::GlobalObject);
        // ...Then the declared WebIDL arguments...
        result.extend(args.into_iter().map(|arg| CPPArgument::In(arg)));
        // ...Then the out param, depending on the return type.
        if let Some(type_) = self
            .webidl_return_type()
            .filter(|type_| type_.needs_out_param())
        {
            result.push(CPPArgument::Out(type_));
        }
        // ...And finally, the `ErrorResult` to throw errors.
        if self.throws().is_some() {
            result.push(CPPArgument::ErrorResult);
        }
        result
    }

    fn cpp_return_type(&self) -> Option<WebIDLType<'_>> {
        self.webidl_return_type()
            .filter(|type_| !type_.needs_out_param())
    }

    fn cpp_return_by(&self) -> ReturnBy<'_> {
        self.webidl_return_type()
            .map(ReturnBy::from_return_type)
            .unwrap_or(ReturnBy::Void)
    }

    fn cpp_throw_by(&self) -> ThrowBy {
        if self.throws().is_some() {
            ThrowBy::ErrorResult("aRv")
        } else {
            ThrowBy::Assert
        }
    }
}

/// Extensions to support WebIDL interface constructors.
pub trait ConstructorExt {
    /// Returns a list of arguments to declare for this constructor in the C++
    /// implementation, including any extras and out parameters.
    fn cpp_arguments(&self) -> Vec<CPPArgument<'_>>;

    /// Indicates how this constructor throws errors, either by an `ErrorResult`
    /// parameter, or by a fatal assertion.
    fn cpp_throw_by(&self) -> ThrowBy;
}

impl ConstructorExt for Constructor {
    fn cpp_arguments(&self) -> Vec<CPPArgument<'_>> {
        let args = self.arguments();
        let mut result = Vec::with_capacity(args.len() + 2);
        // First the `GlobalObject`, just like for static methods...
        result.push(CPPArgument::GlobalObject);
        result.extend(args.into_iter().map(|arg| CPPArgument::In(arg)));
        // Constructors never take out params, since they must return an
        // instance of the object.
        if self.throws().is_some() {
            // ...But they can throw, so pass an `ErrorResult` if we need to
            // throw errors.
            result.push(CPPArgument::ErrorResult);
        }
        result
    }

    fn cpp_throw_by(&self) -> ThrowBy {
        if self.throws().is_some() {
            ThrowBy::ErrorResult("aRv")
        } else {
            ThrowBy::Assert
        }
    }
}

/// Extensions to support WebIDL interface methods.
pub trait MethodExt {
    /// Returns the WebIDL return type of this method.
    fn webidl_return_type(&self) -> Option<WebIDLType<'_>>;

    /// Returns a list of arguments to declare for this method in the C++
    /// implementation, including any extras and out parameters.
    fn cpp_arguments(&self) -> Vec<CPPArgument<'_>>;

    /// Returns the C++ return type of this function, or `None` if the method
    /// doesn't return a value, or returns it via an out parameter.
    fn cpp_return_type(&self) -> Option<WebIDLType<'_>>;

    /// Indicates how this function returns its result, either by value or via
    /// an out parameter.
    fn cpp_return_by(&self) -> ReturnBy<'_>;

    /// Indicates how this method throws errors, either by an `ErrorResult`
    /// parameter, or by a fatal assertion.
    fn cpp_throw_by(&self) -> ThrowBy;
}

impl MethodExt for Method {
    fn webidl_return_type(&self) -> Option<WebIDLType<'_>> {
        self.return_type().map(WebIDLType::from)
    }

    fn cpp_arguments(&self) -> Vec<CPPArgument<'_>> {
        let args = self.arguments();
        let mut result = Vec::with_capacity(args.len() + 2);
        // Methods don't take a `GlobalObject` as their first argument.
        result.extend(args.into_iter().map(|arg| CPPArgument::In(arg)));
        if let Some(type_) = self
            .webidl_return_type()
            .filter(|type_| type_.needs_out_param())
        {
            // ...But they can take out params, since they return values.
            result.push(CPPArgument::Out(type_));
        }
        if self.throws().is_some() {
            // ...And they can throw.
            result.push(CPPArgument::ErrorResult);
        }
        result
    }

    fn cpp_return_type(&self) -> Option<WebIDLType<'_>> {
        self.webidl_return_type()
            .filter(|type_| !type_.needs_out_param())
    }

    fn cpp_return_by(&self) -> ReturnBy<'_> {
        self.webidl_return_type()
            .map(ReturnBy::from_return_type)
            .unwrap_or(ReturnBy::Void)
    }

    fn cpp_throw_by(&self) -> ThrowBy {
        if self.throws().is_some() {
            ThrowBy::ErrorResult("aRv")
        } else {
            ThrowBy::Assert
        }
    }
}

/// Extensions to support WebIDL static method, constructor, and interface
/// method arguments.
pub trait ArgumentExt {
    /// Returns the argument type.
    fn webidl_type(&self) -> WebIDLType<'_>;

    /// Indicates if the argument should have an `optional` keyword. `true`
    /// for nullable dictionaries and arguments that declare a default value
    /// in UDL; `false` otherwise.
    fn optional(&self) -> bool;

    /// Returns the default value for this argument, if one is specified.
    fn webidl_default_value(&self) -> Option<Literal>;
}

impl ArgumentExt for Argument {
    fn webidl_type(&self) -> WebIDLType<'_> {
        self.type_().into()
    }

    fn optional(&self) -> bool {
        if self.webidl_default_value().is_some() {
            return true;
        }
        false
    }

    fn webidl_default_value(&self) -> Option<Literal> {
        if let Some(literal) = self.default_value() {
            return Some(literal);
        }
        if self.webidl_type().is_optional_record() {
            // Nullable UDL dictionaries must declare a default value
            // in WebIDL.
            return Some(Literal::EmptyMap);
        }
        None
    }
}

/// Extensions to support WebIDL dictionary members.
pub trait FieldExt {
    /// Returns the member type.
    fn webidl_type(&self) -> WebIDLType<'_>;

    /// Indicates if the member should have a `required` keyword. In UDL, all
    /// dictionary members are required by default; in WebIDL, they're optional.
    /// For WebIDL, all members are `required`, except for nullable
    /// dictionaries, which must be optional.
    fn required(&self) -> bool;

    /// Returns the default value for this member, if one is defined.
    fn webidl_default_value(&self) -> Option<Literal>;
}

impl FieldExt for Field {
    fn webidl_type(&self) -> WebIDLType<'_> {
        match self.type_() {
            Type::Optional(inner) => WebIDLType::Optional(Box::new((&**inner).into())),
            type_ => type_.into(),
        }
    }

    fn required(&self) -> bool {
        !matches!(self.type_(), Type::Optional(_))
    }

    fn webidl_default_value(&self) -> Option<Literal> {
        if self.webidl_type().is_optional_record() {
            // Nullable UDL dictionaries must declare a default value
            // in WebIDL.
            return Some(Literal::EmptyMap);
        }
        None
    }
}

/// Describes how a function returns its result.
pub enum ReturnBy<'a> {
    /// The function returns its result in an out parameter with the given
    /// name and type.
    OutParam(&'static str, WebIDLType<'a>),

    /// The function returns its result by value.
    Value(WebIDLType<'a>),

    /// The function doesn't declare a return type.
    Void,
}

impl<'a> ReturnBy<'a> {
    fn from_return_type(type_: WebIDLType<'a>) -> Self {
        if type_.needs_out_param() {
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
pub enum CPPArgument<'a> {
    /// The argument is a `GlobalObject`, passed to constructors, static, and
    /// namespace methods.
    GlobalObject,

    /// The argument is an `ErrorResult`, passed to throwing functions.
    ErrorResult,

    /// The argument is a normal input parameter.
    In(&'a Argument),

    /// The argument is an out parameter used to return results by reference.
    Out(WebIDLType<'a>),
}

impl<'a> CPPArgument<'a> {
    /// Returns the argument name.
    pub fn name(&self) -> &'a str {
        match self {
            CPPArgument::GlobalObject => "aGlobal",
            CPPArgument::ErrorResult => "aRv",
            CPPArgument::In(arg) => arg.name(),
            CPPArgument::Out(_) => "aRetVal",
        }
    }
}
