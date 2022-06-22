/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::func_names;
use crate::interface;
/// Extension traits for interface items
use bindings_ir::ir::*;
use extend::ext;

/// Toolkit of internal functions, like lift/lower

#[ext]
pub(super) impl interface::Type {
    fn call_lift(&self, value: Expression) -> Expression {
        call(func_names::lift(self), [value])
    }

    fn call_lower(&self, value: Expression) -> Expression {
        call(func_names::lower(self), [value])
    }

    fn call_allocation_size(&self, value: Expression) -> Expression {
        call(func_names::allocation_size(self), [value])
    }

    fn call_read(&self, value: Expression) -> Expression {
        call(func_names::read(self), [value])
    }

    fn call_write(&self, buf: Expression, value: Expression) -> Expression {
        call(func_names::write(self), [buf, value])
    }
}

#[ext]
pub(super) impl interface::FFIType {
    /// Get the IR type used when lifting types received across the FFI
    fn ir_lift_type(&self) -> Type {
        match self {
            Self::UInt8 => uint8(),
            Self::Int8 => int8(),
            Self::UInt16 => uint16(),
            Self::Int16 => int16(),
            Self::UInt32 => uint32(),
            Self::Int32 => int32(),
            Self::UInt64 => uint64(),
            Self::Int64 => int64(),
            Self::Float32 => float32(),
            Self::Float64 => float64(),
            // Rust buffers are passed in as an owned CStruct
            Self::RustBuffer => cstruct("RustBuffer"),
            // Objects are received in an owned pointer
            interface::FFIType::RustArcPtr(name) => pointer(name),
            _ => unimplemented!("{:?}", self),
        }
    }

    /// Get the IR type used when lowering this type to send across the FFI
    fn ir_lower_type(&self) -> Type {
        match self {
            // The only difference between lower and lift types is that lowered pointers are passed
            // as a reference rather than an owned pointer.  The UDL doesn't specify mutable or
            // not, so we need to assume it here.
            Self::RustArcPtr(name) => reference_mut(pointer(name)),
            _ => self.ir_lift_type(),
        }
    }
}
