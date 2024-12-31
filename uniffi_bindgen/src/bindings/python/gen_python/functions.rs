/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use uniffi_internal_macros::{AsCallable, AsType};

use super::{AsType, Type};
use crate::interface::ir::CallableKind;

/// Represents a standalone function.
///
/// Each `Function` corresponds to a standalone function in the rust module,
/// and has a corresponding standalone function in the foreign language bindings.
///
/// In the FFI, this will be a standalone function with appropriately lowered types.
#[derive(Debug, Clone, AsCallable)]
pub struct Function {
    pub name: String,
    pub docstring: Option<String>,
    pub callable: Callable,
}

#[derive(Debug, Clone)]
pub struct Callable {
    pub kind: CallableKind,
    pub async_data: Option<AsyncData>,
    pub arguments: Vec<Argument>,
    pub return_type: Option<Type>,
    pub throws_type: Option<Type>,
    pub ffi_func: String,
}

#[derive(Debug, Clone)]
pub struct AsyncData {
    pub ffi_rust_future_poll: String,
    pub ffi_rust_future_complete: String,
    pub ffi_rust_future_free: String,
    /// The FFI struct to pass to the completion function for callback interface methods
    pub foreign_future_result_type: String,
}

/// Represents an argument to a function/constructor/method call.
///
/// Each argument has a name and a type, along with some optional metadata.
#[derive(Debug, Clone, AsType)]
pub struct Argument {
    pub name: String,
    pub ty: Type,
    pub default: Option<String>,
}

/// Function/Method/Constructor node that can be mapped to a Callable
pub trait AsCallable {
    fn as_callable(&self) -> &Callable;

    fn async_data(&self) -> Option<&AsyncData> {
        self.as_callable().async_data.as_ref()
    }

    fn arguments(&self) -> &[Argument] {
        &self.as_callable().arguments
    }

    fn return_type(&self) -> Option<&Type> {
        self.as_callable().return_type.as_ref()
    }

    fn throws_type(&self) -> Option<&Type> {
        self.as_callable().throws_type.as_ref()
    }

    fn ffi_func(&self) -> &str {
        &self.as_callable().ffi_func
    }

    fn is_async(&self) -> bool {
        self.as_callable().async_data.is_some()
    }

    fn is_sync(&self) -> bool {
        !self.is_async()
    }

    fn is_function(&self) -> bool {
        matches!(self.as_callable().kind, CallableKind::Function)
    }

    fn is_constructor(&self) -> bool {
        matches!(self.as_callable().kind, CallableKind::Constructor { .. })
    }

    fn is_primary_constructor(&self) -> bool {
        matches!(
            self.as_callable().kind,
            CallableKind::Constructor { primary: true, .. }
        )
    }

    fn is_alternate_constructor(&self) -> bool {
        matches!(
            self.as_callable().kind,
            CallableKind::Constructor { primary: false, .. }
        )
    }

    fn is_method(&self) -> bool {
        matches!(
            self.as_callable().kind,
            CallableKind::Method { .. } | CallableKind::VTableMethod { .. }
        )
    }
}

impl AsCallable for Callable {
    fn as_callable(&self) -> &Callable {
        self
    }
}

impl<T: AsCallable> AsCallable for &T {
    fn as_callable(&self) -> &Callable {
        (**self).as_callable()
    }
}
