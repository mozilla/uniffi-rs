/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::{FfiFunctionRef, FfiType, Literal, Type};

/// Represents a standalone function.
///
/// Each `Function` corresponds to a standalone function in the rust module,
/// and has a corresponding standalone function in the foreign language bindings.
///
/// In the FFI, this will be a standalone function with appropriately lowered types.
#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub docstring: Option<String>,
    pub module_path: String,
    pub callable: Callable,
}

/// Callable data, this is shared between functions, methods, and constructors.
#[derive(Debug, Clone)]
pub struct Callable {
    pub kind: CallableKind,
    pub async_data: Option<AsyncData>,
    pub arguments: Vec<Argument>,
    pub return_type: Option<Type>,
    pub throws_type: Option<Type>,
    pub ffi_func: FfiFunctionRef,
}

#[derive(Debug, Clone)]
pub enum CallableKind {
    Function,
    Method,
    VTableMethod,
    Constructor { primary: bool },
}

#[derive(Debug, Clone)]
pub struct AsyncData {
    pub ffi_rust_future_poll: FfiFunctionRef,
    pub ffi_rust_future_complete: FfiFunctionRef,
    pub ffi_rust_future_free: FfiFunctionRef,
    /// The FFI struct to pass to the completion function for callback interface methods
    pub foreign_future_result_type: FfiType,
}

/// Represents an argument to a function/constructor/method call.
///
/// Each argument has a name and a type, along with some optional metadata.
#[derive(Debug, Clone)]
pub struct Argument {
    pub name: String,
    pub ty: Type,
    pub by_ref: bool,
    pub optional: bool,
    pub default: Option<Literal>,
}
