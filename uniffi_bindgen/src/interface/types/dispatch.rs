/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Implement type behavior using the dispatch pattern
//!
//! There are a couple issues with implementing type behavior:
//!
//!    - `Type` has a lot of variants.  It can be hard to implement traits on `Type` when each
//!    function needs to match against them all.
//!    - A secondary issue is that there are types that correspond to `Type` variants (Record,
//!      Object, CallbackInterface, etc.)  It would be nice for these types to share behavior with
//!      their `Type` variant.
//!
//! This module helps solve both issues using the dispatch pattern and macros:
//!
//!  - Define a set of handler structs.  Each one will handle some subset of `Type` variants
//!    (usually 1).
//!  - Define a macro that can dispatch function calls for `Type` to one of the handler structs.
//!  - Wrap traits with the `type_dispatch!` macro, defined here.  `type_dispatch!` derives a trait
//!    impl for `Type`, `Record`, `Object`, `CallbackInterface`, etc. by dispatching the calls to
//!    the appropriate handler type.  See `CodeType` and `KotlinCodeType` for an example.

use super::Type;

// Dispatch handler for simple types that correspond to standard types on the target language.
// It's easier to implement traits for all of these together than with separate structs.
pub enum SimpleTypeHandler {
    UInt8,
    Int8,
    UInt16,
    Int16,
    UInt32,
    Int32,
    UInt64,
    Int64,
    Float32,
    Float64,
    Boolean,
    String,
}

// Other variants are mapped 1-1 to structs.
//
// When handlers need to reference names and other data from the `Type` instance, we use the `'d`
// lifetime.  This means that these references are only valid for the dispatched call.
pub struct RecordTypeHandler<'d> { pub name: &'d str }
pub struct EnumTypeHandler<'d> { pub name: &'d str }
pub struct ErrorTypeHandler<'d> { pub name: &'d str }
pub struct ObjectTypeHandler<'d> { pub name: &'d str }
pub struct CallbackInterfaceTypeHandler<'d> { pub name: &'d str }
pub struct TimestampTypeHandler;
pub struct DurationTypeHandler;
pub struct OptionalTypeHandler<'d> { pub inner: &'d Type }
pub struct SequenceTypeHandler<'d> { pub inner: &'d Type }
pub struct MapTypeHandler<'d> { pub inner: &'d Type }
pub struct ExternalTypeHandler<'d> { pub name: &'d str, pub crate_name: &'d str }
pub struct WrapperTypeHandler<'d> { pub name: &'d str, pub wrapped: &'d Type }

// Dispatch function calls for `Type` to one of the `TypeHandler` structs.
macro_rules! dispatch_type_function(
    ($self:ident, $fn_name:ident, ($($param:ident),*)) => {
        match $self {
            Type::UInt8 => SimpleTypeHandler::UInt8.$fn_name($($param),*),
            Type::Int8 => SimpleTypeHandler::Int8.$fn_name($($param),*),
            Type::UInt16 => SimpleTypeHandler::UInt16.$fn_name($($param),*),
            Type::Int16 => SimpleTypeHandler::Int16.$fn_name($($param),*),
            Type::UInt32 => SimpleTypeHandler::UInt32.$fn_name($($param),*),
            Type::Int32 => SimpleTypeHandler::Int32.$fn_name($($param),*),
            Type::UInt64 => SimpleTypeHandler::UInt64.$fn_name($($param),*),
            Type::Int64 => SimpleTypeHandler::Int64.$fn_name($($param),*),
            Type::Float32 => SimpleTypeHandler::Float32.$fn_name($($param),*),
            Type::Float64 => SimpleTypeHandler::Float64.$fn_name($($param),*),
            Type::Boolean => SimpleTypeHandler::Boolean.$fn_name($($param),*),
            Type::String => SimpleTypeHandler::String.$fn_name($($param),*),
            Type::Timestamp => TimestampTypeHandler.$fn_name($($param),*),
            Type::Duration => DurationTypeHandler.$fn_name($($param),*),
            Type::Object(name) => ObjectTypeHandler { name }.$fn_name($($param),*),
            Type::Record(name) => RecordTypeHandler { name }.$fn_name($($param),*),
            Type::Enum(name) => EnumTypeHandler { name }.$fn_name($($param),*),
            Type::Error(name) => ErrorTypeHandler { name }.$fn_name($($param),*),
            Type::CallbackInterface(name) => CallbackInterfaceTypeHandler {
                name,
            }.$fn_name($($param),*),
            Type::Optional(inner) => OptionalTypeHandler { inner }.$fn_name($($param),*),
            Type::Sequence(inner) => SequenceTypeHandler { inner }.$fn_name($($param),*),
            Type::Map(inner) => MapTypeHandler { inner }.$fn_name($($param),*),
            Type::External { name, crate_name } => ExternalTypeHandler {
                name,
                crate_name,
            }.$fn_name($($param),*),
            Type::Wrapped { name, prim } => WrapperTypeHandler {
                name,
                wrapped: prim.as_ref(),
            }.$fn_name($($param),*),
        }
    }
);

// The `type_dispatch!` trait wrapper.  F

macro_rules! type_dispatch {
    (
        $(#[$meta:meta])*
        $vis:vis trait $name:ident $tt:tt
    ) => {
        $(#[$meta])*
        $vis trait $name $tt

        type_dispatch_impl_for_type!($tt, $name);
        type_dispatch_impl_for_related!(crate::interface::Record, $tt, $name);
        type_dispatch_impl_for_related!(crate::interface::Enum, $tt, $name);
        type_dispatch_impl_for_related!(crate::interface::CallbackInterface, $tt, $name);
        type_dispatch_impl_for_related!(crate::interface::Error, $tt, $name);
        type_dispatch_impl_for_related!(crate::interface::Object, $tt, $name);
    };
    (
        $(#[$meta:meta])*
        $vis:vis trait $name:ident : $super:ident $tt:tt
    ) => {
        $(#[$meta])*
        $vis trait $name : $super $tt

        type_dispatch_impl_for_type!($tt, $name);
        type_dispatch_impl_for_related!(crate::interface::Record, $tt, $name);
        type_dispatch_impl_for_related!(crate::interface::Enum, $tt, $name);
        type_dispatch_impl_for_related!(crate::interface::CallbackInterface, $tt, $name);
        type_dispatch_impl_for_related!(crate::interface::Error, $tt, $name);
        type_dispatch_impl_for_related!(crate::interface::Object, $tt, $name);
    }
}

macro_rules! type_dispatch_impl_for_type (
    (
        {
            $(
                $(#[$fn_meta:meta])*
                $vis:vis fn $fn_name:ident(&self $(, $param:ident : $type:ty)* ) $(-> $fn_return:ty)? $({ $($stmt:stmt)* })?$(;)?
            )+
        },
        $trait_name:ident
    ) => {
        impl $trait_name for crate::interface::Type {
            $(
                $vis fn $fn_name(&self $(, $param: $type)* ) $(-> $fn_return)? {
                    use crate::interface::types::Type;
                    use crate::interface::types::dispatch::*;
                    dispatch_type_function!(self, $fn_name, ($($param),*))
                }
            )+
        }
    }
);

macro_rules! type_dispatch_impl_for_related (
    (
        $other_type:path,
        {
            $(
                $(#[$fn_meta:meta])*
                $vis:vis fn $fn_name:ident(&self $(, $param:ident : $type:ty)* ) $(-> $fn_return:ty)? $({ $($stmt:stmt)* })?$(;)?
            )+
        },
        $trait_name:ident
    ) => {
        impl $trait_name for $other_type {
            $(
                $vis fn $fn_name(&self $(, $param: $type)* ) $(-> $fn_return)? {
                    self.type_().$fn_name($($param),*)
                }
            )+
        }
    }
);
