/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::ext::*;
use super::util::arg_call_status;
/// IntoIr trait -- Convert ComponentInterface items to bindings-ir items
///
/// IntoIr is basically Into<> trait, but using a local trait allows us to define in on things like
/// Option<T>.
use crate::interface;
use bindings_ir::ir::*;
use std::iter::once;

pub trait IntoIr {
    type IrType;
    fn into_ir(self) -> Self::IrType;
}

impl IntoIr for interface::Type {
    type IrType = Type;

    fn into_ir(self) -> Self::IrType {
        match self {
            Self::UInt8 => Type::UInt8,
            Self::Int8 => Type::Int8,
            Self::UInt16 => Type::UInt16,
            Self::Int16 => Type::Int16,
            Self::UInt32 => Type::UInt32,
            Self::Int32 => Type::Int32,
            Self::UInt64 => Type::UInt64,
            Self::Int64 => Type::Int64,
            Self::Float32 => Type::Float32,
            Self::Float64 => Type::Float64,
            Self::Boolean => Type::Boolean,
            Self::String => Type::String,
            Self::Enum(name) => Type::Object { class: name.into() },
            Self::Object(name) => Type::Object { class: name.into() },
            Self::Record(name) => Type::Object { class: name.into() },
            Self::Error(name) => Type::Object { class: name.into() },
            Self::Optional(inner) => Type::Nullable {
                inner: Box::new(inner.into_ir()),
            },
            Self::Sequence(inner) => Type::List {
                inner: Box::new(inner.into_ir()),
            },
            Self::Map(key, value) => Type::Map {
                key: Box::new(key.into_ir()),
                value: Box::new(value.into_ir()),
            },
            Self::Timestamp => unimplemented!("Type::Timestamp"),
            Self::Duration => unimplemented!("Type::Duration"),
            Self::CallbackInterface(_) => unimplemented!("Type::CallbackInterface"),
            Self::External { .. } => unimplemented!("Type::External"),
            Self::Custom { .. } => unimplemented!("Type::Custom"),
        }
    }
}

impl IntoIr for interface::Argument {
    type IrType = Argument;

    fn into_ir(self) -> Self::IrType {
        arg(self.name(), self.type_().into_ir())
    }
}

impl IntoIr for Vec<&interface::Argument> {
    type IrType = Vec<Argument>;

    fn into_ir(self) -> Self::IrType {
        self.into_iter().cloned().map(IntoIr::into_ir).collect()
    }
}

impl IntoIr for interface::Field {
    type IrType = Field;

    fn into_ir(self) -> Self::IrType {
        field(self.name(), self.type_().into_ir())
    }
}

impl IntoIr for Vec<&interface::Field> {
    type IrType = Vec<Field>;

    fn into_ir(self) -> Self::IrType {
        self.into_iter().cloned().map(IntoIr::into_ir).collect()
    }
}

impl IntoIr for Option<&interface::Type> {
    type IrType = Option<Type>;

    fn into_ir(self) -> Self::IrType {
        self.cloned().map(interface::Type::into_ir)
    }
}

impl IntoIr for &interface::FFIFunction {
    type IrType = FFIFunctionDef;

    fn into_ir(self) -> Self::IrType {
        FFIFunctionDef {
            name: self.name().into(),
            args: self
                .arguments()
                .into_iter()
                .map(|a| arg(a.name(), a.type_().ir_lower_type()))
                .chain(once(arg_call_status()))
                .collect(),
            return_type: self.return_type().map(|type_| type_.ir_lift_type()),
        }
    }
}
