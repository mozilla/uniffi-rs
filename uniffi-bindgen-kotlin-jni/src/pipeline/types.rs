/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::fmt::Display;

use super::*;

pub fn map_type_node(type_node: general::TypeNode, context: &Context) -> Result<TypeNode> {
    let ty = &type_node.ty;

    let ffi_types = context.ffi_type_oracle.get_ffi_types(ty)?;

    Ok(TypeNode {
        is_used_as_error: type_node.is_used_as_error,
        id: type_node.id,
        type_rs: type_rs(ty, context)?,
        type_kt: type_kt(ty, context)?,
        ty: type_node.ty.map_node(context)?,
        ffi_types,
    })
}

pub fn type_rs(ty: &Type, context: &Context) -> Result<String> {
    Ok(match ty {
        Type::UInt8 => "::std::primitive::u8".into(),
        Type::Int8 => "::std::primitive::i8".into(),
        Type::UInt16 => "::std::primitive::u16".into(),
        Type::Int16 => "::std::primitive::i16".into(),
        Type::UInt32 => "::std::primitive::u32".into(),
        Type::Int32 => "::std::primitive::i32".into(),
        Type::UInt64 => "::std::primitive::u64".into(),
        Type::Int64 => "::std::primitive::i64".into(),
        Type::Float32 => "::std::primitive::f32".into(),
        Type::Float64 => "::std::primitive::f64".into(),
        Type::Boolean => "::std::primitive::bool".into(),
        Type::String => "::std::string::String".into(),
        Type::Optional { inner_type } => {
            format!("::std::option::Option::<{}>", type_rs(inner_type, context)?)
        }
        Type::Sequence { inner_type } => {
            format!("::std::vec::Vec::<{}>", type_rs(inner_type, context)?)
        }
        Type::Map {
            key_type,
            value_type,
        } => {
            format!(
                "::std::collections::HashMap::<{}, {}>",
                type_rs(key_type, context)?,
                type_rs(value_type, context)?,
            )
        }
        Type::Set { inner_type } => {
            format!(
                "::std::collections::HashSet::<{}>",
                type_rs(inner_type, context)?,
            )
        }
        Type::Record { orig_name, .. }
        | Type::Enum { orig_name, .. }
        | Type::Custom { orig_name, .. } => {
            format!("{}::{orig_name}", context.rust_module_path_for_type(ty)?)
        }
        Type::Interface { orig_name, .. } => {
            format!(
                "::std::sync::Arc<{}::{orig_name}>",
                context.rust_module_path_for_type(ty)?
            )
        }
        _ => todo!(),
    })
}

pub fn type_kt(ty: &Type, context: &Context) -> Result<String> {
    Ok(match ty {
        Type::UInt8 => "kotlin.UByte".into(),
        Type::Int8 => "kotlin.Byte".into(),
        Type::UInt16 => "kotlin.UShort".into(),
        Type::Int16 => "kotlin.Short".into(),
        Type::UInt32 => "kotlin.UInt".into(),
        Type::Int32 => "kotlin.Int".into(),
        Type::UInt64 => "kotlin.ULong".into(),
        Type::Int64 => "kotlin.Long".into(),
        Type::Float32 => "kotlin.Float".into(),
        Type::Float64 => "kotlin.Double".into(),
        Type::Boolean => "kotlin.Boolean".into(),
        Type::String => "kotlin.String".into(),
        Type::Optional { inner_type } => {
            format!("{}?", type_kt(inner_type, context)?)
        }
        Type::Sequence { inner_type } => {
            format!("kotlin.collections.List<{}>", type_kt(inner_type, context)?)
        }
        Type::Map {
            key_type,
            value_type,
        } => {
            format!(
                "kotlin.collections.Map<{}, {}>",
                type_kt(key_type, context)?,
                type_kt(value_type, context)?,
            )
        }
        Type::Set { inner_type } => {
            format!("kotlin.collections.Set<{}>", type_kt(inner_type, context)?)
        }
        Type::Record {
            namespace, name, ..
        }
        | Type::Enum {
            namespace, name, ..
        }
        | Type::Custom {
            namespace, name, ..
        }
        | Type::Interface {
            namespace, name, ..
        } => format!(
            "{}.{}",
            context.package_name_for_namespace(namespace)?,
            names::class_name_kt(name, context.types_used_as_error.contains(ty)),
        ),
        _ => todo!(),
    })
}

impl TypeNode {
    /// Function to lower this type
    ///
    /// This converts the high-level type into one or more FfiTypes.
    /// If this type has exactly 1 FFI type, then this returns that type directly.
    /// Otherwise it returns a tuple.
    pub fn lower_fn_rs(&self) -> String {
        let id = self.id;
        match &self.ty {
            Type::Int8 => "uniffi_jni::lower_i8".into(),
            Type::Int16 => "uniffi_jni::lower_i16".into(),
            Type::Int32 => "uniffi_jni::lower_i32".into(),
            Type::Int64 => "uniffi_jni::lower_i64".into(),
            Type::UInt8 => "uniffi_jni::lower_u8".into(),
            Type::UInt16 => "uniffi_jni::lower_u16".into(),
            Type::UInt32 => "uniffi_jni::lower_u32".into(),
            Type::UInt64 => "uniffi_jni::lower_u64".into(),
            Type::Float32 => "uniffi_jni::lower_f32".into(),
            Type::Float64 => "uniffi_jni::lower_f64".into(),
            Type::Boolean => "uniffi_jni::lower_bool".into(),
            Type::String => "uniffi_jni::lower_string".into(),
            Type::Bytes => "uniffi_jni::lower_bytes".into(),
            Type::Optional { inner_type } => match &**inner_type {
                Type::Boolean => "uniffi_jni::lower_option_bool".into(),
                Type::Int8 => "uniffi_jni::lower_option_i8".into(),
                Type::UInt8 => "uniffi_jni::lower_option_u8".into(),
                Type::Int16 => "uniffi_jni::lower_option_i16".into(),
                Type::UInt16 => "uniffi_jni::lower_option_u16".into(),
                Type::Int32 => "uniffi_jni::lower_option_i32".into(),
                Type::UInt32 => "uniffi_jni::lower_option_u32".into(),
                Type::Float32 => "uniffi_jni::lower_option_f32".into(),
                Type::Float64 => "uniffi_jni::lower_option_f64".into(),
                Type::String => "uniffi_jni::lower_option_string".into(),
                _ => format!("lower_type_{id}"),
            },
            Type::Sequence { inner_type } => match &**inner_type {
                Type::Int8 => "uniffi_jni::lower_vec_i8".into(),
                Type::UInt8 => "uniffi_jni::lower_vec_u8".into(),
                Type::Int16 => "uniffi_jni::lower_vec_i16".into(),
                Type::UInt16 => "uniffi_jni::lower_vec_u16".into(),
                Type::Int32 => "uniffi_jni::lower_vec_i32".into(),
                Type::UInt32 => "uniffi_jni::lower_vec_u32".into(),
                Type::Int64 => "uniffi_jni::lower_vec_i64".into(),
                Type::UInt64 => "uniffi_jni::lower_vec_u64".into(),
                Type::Float32 => "uniffi_jni::lower_vec_f32".into(),
                Type::Float64 => "uniffi_jni::lower_vec_f64".into(),
                _ => format!("lower_type_{id}"),
            },
            _ => format!("lower_type_{id}"),
        }
    }

    /// Function to lift this type
    ///
    /// This converts one or more FfiTypes into the high-level type.
    pub fn lift_fn_rs(&self) -> String {
        let id = self.id;
        match &self.ty {
            Type::Int8 => "uniffi_jni::lift_i8".into(),
            Type::Int16 => "uniffi_jni::lift_i16".into(),
            Type::Int32 => "uniffi_jni::lift_i32".into(),
            Type::Int64 => "uniffi_jni::lift_i64".into(),
            Type::UInt8 => "uniffi_jni::lift_u8".into(),
            Type::UInt16 => "uniffi_jni::lift_u16".into(),
            Type::UInt32 => "uniffi_jni::lift_u32".into(),
            Type::UInt64 => "uniffi_jni::lift_u64".into(),
            Type::Float32 => "uniffi_jni::lift_f32".into(),
            Type::Float64 => "uniffi_jni::lift_f64".into(),
            Type::Boolean => "uniffi_jni::lift_bool".into(),
            Type::String => "uniffi_jni::lift_string".into(),
            Type::Bytes => "uniffi_jni::lift_bytes".into(),
            Type::Optional { inner_type } => match &**inner_type {
                Type::Boolean => "uniffi_jni::lift_option_bool".into(),
                Type::Int8 => "uniffi_jni::lift_option_i8".into(),
                Type::UInt8 => "uniffi_jni::lift_option_u8".into(),
                Type::Int16 => "uniffi_jni::lift_option_i16".into(),
                Type::UInt16 => "uniffi_jni::lift_option_u16".into(),
                Type::Int32 => "uniffi_jni::lift_option_i32".into(),
                Type::UInt32 => "uniffi_jni::lift_option_u32".into(),
                Type::Float32 => "uniffi_jni::lift_option_f32".into(),
                Type::Float64 => "uniffi_jni::lift_option_f64".into(),
                Type::String => "uniffi_jni::lift_option_string".into(),
                _ => format!("lift_type_{id}"),
            },
            Type::Sequence { inner_type } => match &**inner_type {
                Type::Int8 => "uniffi_jni::lift_vec_i8".into(),
                Type::UInt8 => "uniffi_jni::lift_vec_u8".into(),
                Type::Int16 => "uniffi_jni::lift_vec_i16".into(),
                Type::UInt16 => "uniffi_jni::lift_vec_u16".into(),
                Type::Int32 => "uniffi_jni::lift_vec_i32".into(),
                Type::UInt32 => "uniffi_jni::lift_vec_u32".into(),
                Type::Int64 => "uniffi_jni::lift_vec_i64".into(),
                Type::UInt64 => "uniffi_jni::lift_vec_u64".into(),
                Type::Float32 => "uniffi_jni::lift_vec_f32".into(),
                Type::Float64 => "uniffi_jni::lift_vec_f64".into(),
                _ => format!("lift_type_{id}"),
            },
            _ => format!("lift_type_{id}"),
        }
    }

    /// Function to write this type from a FFI buffer
    pub fn write_fn_rs(&self) -> String {
        let id = self.id;
        match &self.ty {
            Type::Int8 => "uniffi::ffibuffer::write_i8".into(),
            Type::Int16 => "uniffi::ffibuffer::write_i16".into(),
            Type::Int32 => "uniffi::ffibuffer::write_i32".into(),
            Type::Int64 => "uniffi::ffibuffer::write_i64".into(),
            Type::UInt8 => "uniffi::ffibuffer::write_u8".into(),
            Type::UInt16 => "uniffi::ffibuffer::write_u16".into(),
            Type::UInt32 => "uniffi::ffibuffer::write_u32".into(),
            Type::UInt64 => "uniffi::ffibuffer::write_u64".into(),
            Type::Float32 => "uniffi::ffibuffer::write_f32".into(),
            Type::Float64 => "uniffi::ffibuffer::write_f64".into(),
            Type::Boolean => "uniffi::ffibuffer::write_bool".into(),
            Type::String => "uniffi::ffibuffer::write_string".into(),
            _ => format!("write_type_{id}"),
        }
    }

    /// Function to read this type from a FFI buffer
    pub fn read_fn_rs(&self) -> String {
        let id = self.id;
        match &self.ty {
            Type::Int8 => "uniffi::ffibuffer::read_i8".into(),
            Type::Int16 => "uniffi::ffibuffer::read_i16".into(),
            Type::Int32 => "uniffi::ffibuffer::read_i32".into(),
            Type::Int64 => "uniffi::ffibuffer::read_i64".into(),
            Type::UInt8 => "uniffi::ffibuffer::read_u8".into(),
            Type::UInt16 => "uniffi::ffibuffer::read_u16".into(),
            Type::UInt32 => "uniffi::ffibuffer::read_u32".into(),
            Type::UInt64 => "uniffi::ffibuffer::read_u64".into(),
            Type::Float32 => "uniffi::ffibuffer::read_f32".into(),
            Type::Float64 => "uniffi::ffibuffer::read_f64".into(),
            Type::Boolean => "uniffi::ffibuffer::read_bool".into(),
            Type::String => "uniffi::ffibuffer::read_string".into(),
            _ => format!("read_type_{id}"),
        }
    }

    /// Function to lower this type
    ///
    /// This converts the high-level type into one or more FfiTypes.
    /// If this type has exactly 1 FFI type, then this returns that type directly.
    /// Otherwise it returns a tuple-like class named `self.lower_type_kt`.
    pub fn lower_fn_kt(&self) -> String {
        let id = self.id;
        match &self.ty {
            Type::Int8 => "lowerByte".into(),
            Type::Int16 => "lowerShort".into(),
            Type::Int32 => "lowerInt".into(),
            Type::Int64 => "lowerLong".into(),
            Type::UInt8 => "lowerUByte".into(),
            Type::UInt16 => "lowerUShort".into(),
            Type::UInt32 => "lowerUInt".into(),
            Type::UInt64 => "lowerULong".into(),
            Type::Float32 => "lowerFloat".into(),
            Type::Float64 => "lowerDouble".into(),
            Type::Boolean => "lowerBoolean".into(),
            Type::String => "lowerString".into(),
            Type::Bytes => "lowerBytes".into(),
            Type::Optional { inner_type } => match &**inner_type {
                Type::Boolean => "lowerOptionBoolean".into(),
                Type::Int8 => "lowerOptionByte".into(),
                Type::UInt8 => "lowerOptionUByte".into(),
                Type::Int16 => "lowerOptionShort".into(),
                Type::UInt16 => "lowerOptionUShort".into(),
                Type::Int32 => "lowerOptionInt".into(),
                Type::UInt32 => "lowerOptionUInt".into(),
                Type::Float32 => "lowerOptionFloat".into(),
                Type::Float64 => "lowerOptionDouble".into(),
                Type::String => "lowerOptionString".into(),
                _ => format!("lowerType{id}"),
            },
            Type::Sequence { inner_type } => match &**inner_type {
                Type::Int8 => "lowerVecByte".into(),
                Type::UInt8 => "lowerVecUByte".into(),
                Type::Int16 => "lowerVecShort".into(),
                Type::UInt16 => "lowerVecUShort".into(),
                Type::Int32 => "lowerVecInt".into(),
                Type::UInt32 => "lowerVecUInt".into(),
                Type::Int64 => "lowerVecLong".into(),
                Type::UInt64 => "lowerVecULong".into(),
                Type::Float32 => "lowerVecFloat".into(),
                Type::Float64 => "lowerVecDouble".into(),
                _ => format!("lowerType{id}"),
            },
            _ => format!("lowerType{id}"),
        }
    }

    /// Function to lift this type
    ///
    /// This converts one or more FfiTypes into the high-level type.
    pub fn lift_fn_kt(&self) -> String {
        let id = self.id;
        match &self.ty {
            Type::Int8 => "liftByte".into(),
            Type::Int16 => "liftShort".into(),
            Type::Int32 => "liftInt".into(),
            Type::Int64 => "liftLong".into(),
            Type::UInt8 => "liftUByte".into(),
            Type::UInt16 => "liftUShort".into(),
            Type::UInt32 => "liftUInt".into(),
            Type::UInt64 => "liftULong".into(),
            Type::Float32 => "liftFloat".into(),
            Type::Float64 => "liftDouble".into(),
            Type::Boolean => "liftBoolean".into(),
            Type::String => "liftString".into(),
            Type::Bytes => "liftBytes".into(),
            Type::Optional { inner_type } => match &**inner_type {
                Type::Boolean => "liftOptionBoolean".into(),
                Type::Int8 => "liftOptionByte".into(),
                Type::UInt8 => "liftOptionUByte".into(),
                Type::Int16 => "liftOptionShort".into(),
                Type::UInt16 => "liftOptionUShort".into(),
                Type::Int32 => "liftOptionInt".into(),
                Type::UInt32 => "liftOptionUInt".into(),
                Type::Float32 => "liftOptionFloat".into(),
                Type::Float64 => "liftOptionDouble".into(),
                Type::String => "liftOptionString".into(),
                _ => format!("liftType{id}"),
            },
            Type::Sequence { inner_type } => match &**inner_type {
                Type::Int8 => "liftVecByte".into(),
                Type::UInt8 => "liftVecUByte".into(),
                Type::Int16 => "liftVecShort".into(),
                Type::UInt16 => "liftVecUShort".into(),
                Type::Int32 => "liftVecInt".into(),
                Type::UInt32 => "liftVecUInt".into(),
                Type::Int64 => "liftVecLong".into(),
                Type::UInt64 => "liftVecULong".into(),
                Type::Float32 => "liftVecFloat".into(),
                Type::Float64 => "liftVecDouble".into(),
                _ => format!("liftType{id}"),
            },
            _ => format!("liftType{id}"),
        }
    }

    /// Function to write this type from a FFI buffer
    pub fn write_fn_kt(&self) -> String {
        let id = self.id;
        match &self.ty {
            Type::Int8 => "writeByte".into(),
            Type::Int16 => "writeShort".into(),
            Type::Int32 => "writeInt".into(),
            Type::Int64 => "writeLong".into(),
            Type::UInt8 => "writeUByte".into(),
            Type::UInt16 => "writeUShort".into(),
            Type::UInt32 => "writeUInt".into(),
            Type::UInt64 => "writeULong".into(),
            Type::Float32 => "writeFloat".into(),
            Type::Float64 => "writeDouble".into(),
            Type::Boolean => "writeBoolean".into(),
            Type::String => "writeString".into(),
            _ => format!("writeType{id}"),
        }
    }

    /// Function to read this type from a FFI buffer
    pub fn read_fn_kt(&self) -> String {
        let id = self.id;
        match &self.ty {
            Type::Int8 => "readByte".into(),
            Type::Int16 => "readShort".into(),
            Type::Int32 => "readInt".into(),
            Type::Int64 => "readLong".into(),
            Type::UInt8 => "readUByte".into(),
            Type::UInt16 => "readUShort".into(),
            Type::UInt32 => "readUInt".into(),
            Type::UInt64 => "readULong".into(),
            Type::Float32 => "readFloat".into(),
            Type::Float64 => "readDouble".into(),
            Type::Boolean => "readBoolean".into(),
            Type::String => "readString".into(),
            _ => format!("readType{id}"),
        }
    }

    /// Does this type lower to a single primitive FFI type?
    ///
    /// If so, the lower function will return that type directly instead of a tuple.
    pub fn lowers_to_primitive(&self) -> bool {
        self.ffi_types.len() == 1
    }

    /// Get FFI values for a lowered variable
    ///
    /// Given a variable that comes from the lower function for this type,
    /// this returns (var_name, ffi_type) pairs for all individual FFI values.
    pub fn ffi_values_rs(&self, lowered_var_name: impl Display) -> Vec<(String, FfiType)> {
        let mut ffi_values = vec![];
        if self.lowers_to_primitive() {
            ffi_values.push((lowered_var_name.to_string(), self.ffi_types[0]));
        } else {
            for (i, ffi_type) in self.ffi_types.iter().enumerate() {
                ffi_values.push((format!("{lowered_var_name}.{i}"), *ffi_type))
            }
        }
        ffi_values
    }

    pub fn ffi_values_kt(&self, lowered_var_name: impl Display) -> Vec<(String, FfiType)> {
        let mut ffi_values = vec![];
        if self.lowers_to_primitive() {
            ffi_values.push((lowered_var_name.to_string(), self.ffi_types[0]));
        } else {
            for (i, ffi_type) in self.ffi_types.iter().enumerate() {
                ffi_values.push((format!("{lowered_var_name}.v{i}"), *ffi_type))
            }
        }
        ffi_values
    }

    /// Rust type that the lower function returns
    ///
    /// If this type has exactly 1 FFI type, then this is that type.
    /// Otherwise this is a tuple of FFI types
    pub fn lowered_type_rs(&self) -> String {
        if self.lowers_to_primitive() {
            self.ffi_types[0].type_rs().into()
        } else {
            format!(
                "({})",
                self.ffi_types
                    .iter()
                    .map(FfiType::type_rs)
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        }
    }

    /// Kotlin type that the lower function returns
    ///
    /// If this type has exactly 1 FFI type, then this is that type.
    /// Otherwise this is a tuple-like class where all fields are named `v{index}`
    pub fn lowered_type_kt(&self) -> String {
        if self.lowers_to_primitive() {
            self.ffi_types[0].type_kt().into()
        } else {
            format!("DeconstructedType{}", self.id)
        }
    }

    /// Static Rust variable to call the Kotlin lift function from Rust
    pub fn lift_kt_from_rust_var(&self) -> String {
        format!("UNIFFI_CACHED_LIFT_KT_{}", self.id)
    }

    pub fn lift_fn_jni_signature(&self) -> String {
        let args: String = self
            .ffi_types
            .iter()
            .map(|ffi_type| ffi_type.jni_signature())
            .collect();
        let ret = self.jni_return_signature();
        format!("({args}){ret}")
    }

    fn jni_return_signature(&self) -> String {
        let mut type_name = self.type_kt.as_str();

        if let Some(inner) = type_name.strip_suffix("?") {
            // Some optional types get special-cased.
            // For others, remove the trailing `?`.
            match inner {
                // Optional primitives are passed as the boxed versions of themselves
                "kotlin.Byte" => return "Ljava/lang/Byte;".into(),
                "kotlin.Short" => return "Ljava/lang/Short;".into(),
                "kotlin.Int" => return "Ljava/lang/Int;".into(),
                "kotlin.Long" => return "Ljava/lang/Long;".into(),
                "kotlin.Float" => return "Ljava/lang/Float;".into(),
                "kotlin.Double" => return "Ljava/lang/Double;".into(),
                "kotlin.Boolean" => return "Ljava/lang/Boolean;".into(),
                // The boxed class comes from Kotlin for unsigned types
                "kotlin.UByte" => return "Lkotlin/UByte;".into(),
                "kotlin.UShort" => return "Lkotlin/UShort;".into(),
                "kotlin.UInt" => return "Lkotlin/UInt;".into(),
                "kotlin.ULong" => return "Lkotlin/ULong;".into(),
                inner => type_name = inner,
            }
        }

        match type_name {
            // Primitive types
            "kotlin.Byte" | "kotlin.UByte" => "B".into(),
            "kotlin.Short" | "kotlin.UShort" => "S".into(),
            "kotlin.Int" | "kotlin.UInt" => "I".into(),
            "kotlin.Long" | "kotlin.ULong" => "J".into(),
            "kotlin.Float" => "F".into(),
            "kotlin.Double" => "D".into(),
            "kotlin.Boolean" => "Z".into(),
            // Signatures for generics use the java class and don't include the inner type
            t if t.starts_with("kotlin.collections.List") => "Ljava/util/List;".into(),
            t if t.starts_with("kotlin.collections.Map") => "Ljava/util/Map;".into(),
            t if t.starts_with("kotlin.collections.Set") => "Ljava/util/Set;".into(),
            type_name => format!("L{};", type_name.replace(".", "/").replace("`", "")),
        }
    }

    pub fn is_interface(&self) -> bool {
        matches!(self.ty, Type::Interface { .. })
    }
}
