/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::fmt::Debug;

use super::*;

/// Node in the BindingsIr
pub trait Node: Debug {
    fn lang_data(&self) -> &LanguageData;
}

/// Node associated with a type
pub trait AsType: Node {
    fn as_type(&self) -> &Type;
}

/// Node associated with an FFI type
pub trait AsFfiType: Node {
    fn as_ffi_type(&self) -> &FfiType;
}

/// Callable node
pub trait Callable: Node {
    fn async_data(&self) -> Option<&AsyncData>;
    fn ffi_func(&self) -> &FfiFunctionRef;
    fn arguments(&self) -> &[Argument];
    fn return_type(&self) -> &ReturnType;
    fn throws_type(&self) -> &ThrowsType;
    fn callable_type(&self) -> CallableType;

    fn is_async(&self) -> bool {
        self.async_data().is_some()
    }

    fn is_sync(&self) -> bool {
        !self.is_async()
    }

    fn is_function(&self) -> bool {
        matches!(self.callable_type(), CallableType::Function)
    }

    fn is_constructor(&self) -> bool {
        matches!(self.callable_type(), CallableType::Constructor { .. })
    }

    fn is_primary_constructor(&self) -> bool {
        matches!(
            self.callable_type(),
            CallableType::Constructor { primary: true, .. }
        )
    }

    fn is_alternate_constructor(&self) -> bool {
        matches!(
            self.callable_type(),
            CallableType::Constructor { primary: false, .. }
        )
    }

    fn is_method(&self) -> bool {
        matches!(self.callable_type(), CallableType::Method { .. })
    }
}

/// Type of callable, used to get details about the specific callable when accessed via the
/// `Callable` trait.
#[derive(Debug, Clone)]
pub enum CallableType {
    Function,
    Method,
    Constructor { primary: bool },
}

/// FFI Callable
pub trait FfiCallable: Node {
    fn arguments(&self) -> &[FfiArgument];
    fn return_type(&self) -> &FfiReturnType;
}

// TODO: create a macro to remove all this repetition.

impl Node for Record {
    fn lang_data(&self) -> &LanguageData {
        &self.lang_data
    }
}

impl Node for Enum {
    fn lang_data(&self) -> &LanguageData {
        &self.lang_data
    }
}

impl Node for Interface {
    fn lang_data(&self) -> &LanguageData {
        &self.lang_data
    }
}

impl Node for CallbackInterface {
    fn lang_data(&self) -> &LanguageData {
        &self.lang_data
    }
}

impl Node for CustomType {
    fn lang_data(&self) -> &LanguageData {
        &self.lang_data
    }
}

impl Node for ExternalType {
    fn lang_data(&self) -> &LanguageData {
        &self.lang_data
    }
}

impl Node for Variant {
    fn lang_data(&self) -> &LanguageData {
        &self.lang_data
    }
}

impl Node for Constructor {
    fn lang_data(&self) -> &LanguageData {
        &self.lang_data
    }
}

impl Node for Function {
    fn lang_data(&self) -> &LanguageData {
        &self.lang_data
    }
}

impl Node for Method {
    fn lang_data(&self) -> &LanguageData {
        &self.lang_data
    }
}

impl Node for Field {
    fn lang_data(&self) -> &LanguageData {
        &self.lang_data
    }
}

impl Node for Argument {
    fn lang_data(&self) -> &LanguageData {
        &self.lang_data
    }
}

impl Node for Type {
    fn lang_data(&self) -> &LanguageData {
        &self.lang_data
    }
}

impl Node for ReturnType {
    fn lang_data(&self) -> &LanguageData {
        &self.lang_data
    }
}

impl Node for ThrowsType {
    fn lang_data(&self) -> &LanguageData {
        &self.lang_data
    }
}

impl Node for Literal {
    fn lang_data(&self) -> &LanguageData {
        &self.lang_data
    }
}

impl Node for FfiStruct {
    fn lang_data(&self) -> &LanguageData {
        &self.lang_data
    }
}

impl Node for FfiFunction {
    fn lang_data(&self) -> &LanguageData {
        &self.lang_data
    }
}

impl Node for FfiFunctionType {
    fn lang_data(&self) -> &LanguageData {
        &self.lang_data
    }
}

impl Node for FfiField {
    fn lang_data(&self) -> &LanguageData {
        &self.lang_data
    }
}

impl Node for FfiArgument {
    fn lang_data(&self) -> &LanguageData {
        &self.lang_data
    }
}

impl Node for FfiFunctionRef {
    fn lang_data(&self) -> &LanguageData {
        &self.lang_data
    }
}

impl Node for FfiReturnType {
    fn lang_data(&self) -> &LanguageData {
        &self.lang_data
    }
}

impl Node for FfiType {
    fn lang_data(&self) -> &LanguageData {
        &self.lang_data
    }
}

impl Node for TypeDefinition {
    fn lang_data(&self) -> &LanguageData {
        match self {
            Self::Builtin(t) => t.lang_data(),
            Self::Record(r) => r.lang_data(),
            Self::Enum(e) => e.lang_data(),
            Self::Interface(i) => i.lang_data(),
            Self::CallbackInterface(c) => c.lang_data(),
            Self::Custom(c) => c.lang_data(),
            Self::External(e) => e.lang_data(),
        }
    }
}

impl Node for FfiDefinition {
    fn lang_data(&self) -> &LanguageData {
        match self {
            Self::Struct(s) => s.lang_data(),
            Self::Function(f) => f.lang_data(),
            Self::FunctionType(f) => f.lang_data(),
        }
    }
}

impl AsType for TypeDefinition {
    fn as_type(&self) -> &Type {
        match self {
            Self::Builtin(t) => t,
            Self::Record(r) => r.as_type(),
            Self::Enum(e) => e.as_type(),
            Self::Interface(i) => i.as_type(),
            Self::CallbackInterface(c) => c.as_type(),
            Self::Custom(c) => c.as_type(),
            Self::External(e) => e.as_type(),
        }
    }
}

impl AsType for Record {
    fn as_type(&self) -> &Type {
        &self.self_type
    }
}

impl AsType for Enum {
    fn as_type(&self) -> &Type {
        &self.self_type
    }
}

impl AsType for Argument {
    fn as_type(&self) -> &Type {
        &self.ty
    }
}

impl AsType for Field {
    fn as_type(&self) -> &Type {
        &self.ty
    }
}

impl AsType for Interface {
    fn as_type(&self) -> &Type {
        &self.self_type
    }
}

impl AsType for CallbackInterface {
    fn as_type(&self) -> &Type {
        &self.self_type
    }
}

impl AsType for CustomType {
    fn as_type(&self) -> &Type {
        &self.self_type
    }
}

impl AsType for ExternalType {
    fn as_type(&self) -> &Type {
        &self.self_type
    }
}

impl AsType for Type {
    fn as_type(&self) -> &Type {
        self
    }
}

impl Callable for Function {
    fn async_data(&self) -> Option<&AsyncData> {
        self.async_data.as_ref()
    }

    fn ffi_func(&self) -> &FfiFunctionRef {
        &self.ffi_func
    }

    fn arguments(&self) -> &[Argument] {
        &self.arguments
    }

    fn return_type(&self) -> &ReturnType {
        &self.return_type
    }

    fn throws_type(&self) -> &ThrowsType {
        &self.throws_type
    }

    fn callable_type(&self) -> CallableType {
        CallableType::Function
    }
}

impl Callable for Constructor {
    fn async_data(&self) -> Option<&AsyncData> {
        self.async_data.as_ref()
    }

    fn ffi_func(&self) -> &FfiFunctionRef {
        &self.ffi_func
    }

    fn arguments(&self) -> &[Argument] {
        &self.arguments
    }

    fn return_type(&self) -> &ReturnType {
        &self.return_type
    }

    fn throws_type(&self) -> &ThrowsType {
        &self.throws_type
    }

    fn callable_type(&self) -> CallableType {
        CallableType::Constructor {
            primary: self.primary,
        }
    }
}

impl Callable for Method {
    fn async_data(&self) -> Option<&AsyncData> {
        self.async_data.as_ref()
    }

    fn ffi_func(&self) -> &FfiFunctionRef {
        &self.ffi_func
    }

    fn arguments(&self) -> &[Argument] {
        &self.arguments
    }

    fn return_type(&self) -> &ReturnType {
        &self.return_type
    }

    fn throws_type(&self) -> &ThrowsType {
        &self.throws_type
    }

    fn callable_type(&self) -> CallableType {
        CallableType::Method
    }
}

impl AsFfiType for FfiArgument {
    fn as_ffi_type(&self) -> &FfiType {
        &self.ty
    }
}

impl AsFfiType for FfiField {
    fn as_ffi_type(&self) -> &FfiType {
        &self.ty
    }
}

impl AsFfiType for FfiType {
    fn as_ffi_type(&self) -> &FfiType {
        self
    }
}

impl<N: Node + ?Sized> Node for &N {
    fn lang_data(&self) -> &LanguageData {
        (*self).lang_data()
    }
}

impl<N: Node + ?Sized> Node for Box<N> {
    fn lang_data(&self) -> &LanguageData {
        (**self).lang_data()
    }
}

impl<T: AsType + ?Sized> AsType for &T {
    fn as_type(&self) -> &Type {
        (*self).as_type()
    }
}

impl<T: AsType + ?Sized> AsType for Box<T> {
    fn as_type(&self) -> &Type {
        (**self).as_type()
    }
}

impl<T: AsFfiType + ?Sized> AsFfiType for &T {
    fn as_ffi_type(&self) -> &FfiType {
        (*self).as_ffi_type()
    }
}

impl<T: AsFfiType + ?Sized> AsFfiType for Box<T> {
    fn as_ffi_type(&self) -> &FfiType {
        (**self).as_ffi_type()
    }
}

impl<T: Callable + ?Sized> Callable for &T {
    fn async_data(&self) -> Option<&AsyncData> {
        (**self).async_data()
    }

    fn ffi_func(&self) -> &FfiFunctionRef {
        (*self).ffi_func()
    }

    fn arguments(&self) -> &[Argument] {
        (*self).arguments()
    }

    fn return_type(&self) -> &ReturnType {
        (*self).return_type()
    }

    fn throws_type(&self) -> &ThrowsType {
        (*self).throws_type()
    }

    fn callable_type(&self) -> CallableType {
        (*self).callable_type()
    }
}

impl<T: Callable + ?Sized> Callable for Box<T> {
    fn async_data(&self) -> Option<&AsyncData> {
        (**self).async_data()
    }

    fn ffi_func(&self) -> &FfiFunctionRef {
        (**self).ffi_func()
    }

    fn arguments(&self) -> &[Argument] {
        (**self).arguments()
    }

    fn return_type(&self) -> &ReturnType {
        (**self).return_type()
    }

    fn throws_type(&self) -> &ThrowsType {
        (**self).throws_type()
    }

    fn callable_type(&self) -> CallableType {
        (**self).callable_type()
    }
}

impl FfiCallable for FfiFunction {
    fn arguments(&self) -> &[FfiArgument] {
        &self.arguments
    }

    fn return_type(&self) -> &FfiReturnType {
        &self.return_type
    }
}

impl FfiCallable for FfiFunctionType {
    fn arguments(&self) -> &[FfiArgument] {
        &self.arguments
    }

    fn return_type(&self) -> &FfiReturnType {
        &self.return_type
    }
}

impl<T: FfiCallable> FfiCallable for &T {
    fn arguments(&self) -> &[FfiArgument] {
        (*self).arguments()
    }

    fn return_type(&self) -> &FfiReturnType {
        (*self).return_type()
    }
}

impl<T: FfiCallable> FfiCallable for Box<T> {
    fn arguments(&self) -> &[FfiArgument] {
        (**self).arguments()
    }

    fn return_type(&self) -> &FfiReturnType {
        (**self).return_type()
    }
}
