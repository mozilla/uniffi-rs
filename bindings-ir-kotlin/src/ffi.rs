/* This Source Code Form is subject to the terms of the Mozilla Public
License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use bindings_ir::{ir::Type, Renderer};
use heck::ToUpperCamelCase;
use tera::{to_value, try_get_value, Result, Value};

pub fn setup_renderer(renderer: &mut Renderer) -> Result<()> {
    // These are needed to deal with Kotlin's half-support of unsigned integers, which it inherited
    // from Java.  Kotlin supports unsigned ints at the language level, but many APIs don't
    // including JNA and ByteBuffer.
    //
    // To work around this, we use signed integers for those APIs then convert to unsigned integers
    // when we actually want to use the values and vice-versa when sending values to those APIs.
    // Note that this conversion just changes the type, no bits are altered.
    renderer
        .tera_mut()
        .register_filter("to_ffi_converter", |value: &'_ Value, _: &_| {
            let type_ = try_get_value!("to_ffi_converter", "value", Type, value);
            Ok(match type_ {
                Type::UInt8 => Value::String("toByte".into()),
                Type::UInt16 => Value::String("toShort".into()),
                Type::UInt32 => Value::String("toInt".into()),
                Type::UInt64 => Value::String("toLong".into()),
                _ => Value::Null,
            })
        });
    renderer
        .tera_mut()
        .register_filter("from_ffi_converter", |value: &'_ Value, _: &_| {
            let type_ = try_get_value!("from_ffi_converter", "value", Type, value);
            Ok(match type_ {
                Type::UInt8 => Value::String("toUByte".into()),
                Type::UInt16 => Value::String("toUShort".into()),
                Type::UInt32 => Value::String("toUInt".into()),
                Type::UInt64 => Value::String("toULong".into()),
                _ => Value::Null,
            })
        });
    renderer
        .tera_mut()
        .register_filter("ffi_type", |value: &'_ Value, _: &_| {
            let type_ = try_get_value!("ffi_type", "value", Type, value);
            Ok(match type_ {
                Type::UInt8 => Value::String("Byte".into()),
                Type::UInt16 => Value::String("Short".into()),
                Type::UInt32 => Value::String("Int".into()),
                Type::UInt64 => Value::String("Long".into()),
                Type::Reference { inner, .. } => match *inner {
                    Type::CStruct { .. } => Value::String("com.sun.jna.Pointer".into()),
                    Type::Pointer { .. } => Value::String("com.sun.jna.Pointer".into()),
                    _ => panic!("References to {inner:?} are not supported"),
                },
                // Other values can be rendered as normal using the Type template by returning the
                // value.
                _ => value.clone(),
            })
        });
    renderer
        .tera_mut()
        .register_tester("struct_reference", |value: Option<&'_ Value>, _: &_| {
            let value = match value {
                Some(v) => v,
                None => return Ok(false)
            };
            let type_ = try_get_value!("ffi_type", "value", Type, value);
            Ok(matches!(type_, Type::Reference { inner, .. } if matches!(*inner, Type::CStruct { .. })))
        });
    // Default value to use for FFI Types in CStructs.  These must have a default because JNA
    // requires a no-arg constructor.
    renderer
        .tera_mut()
        .register_filter("ffi_default", |value: &'_ Value, _: &_| {
            let type_ = try_get_value!("ffi_type", "value", Type, value);
            Ok(match type_ {
                Type::Int8 | Type::Int16 | Type::Int32 | Type::Int64 => to_value("0"),
                Type::UInt8 => to_value("0.toUByte()"),
                Type::UInt16 => to_value("0.toUShort()"),
                Type::UInt32 => to_value("0.toUInt()"),
                Type::UInt64 => to_value("0.toULong()"),
                Type::Float32 => to_value("0.0.toFloat()"),
                Type::Float64 => to_value("0.0"),
                Type::Pointer { .. } => to_value("com.sun.jna.Pointer.NULL"),
                Type::CStruct { name } => to_value(format!("{}()", name.to_upper_camel_case())),
                Type::Nullable { .. } => to_value("null"),
                _ => panic!("{type_:?} not supported as a CStruct field"),
            }?)
        });
    renderer.add_ast_templates([
        ("PointerSize", "com.sun.jna.Native.POINTER_SIZE"),
        ("CStructDef", include_str!("templates/CStructDef.kt")),
        ("CStructCreate", "{{ name }}({{ values|comma_join }})"),
        (
            "FFICall",
            "BindgenRendererFFILib.{{ name|to_lower_camel_case }}({{ values|comma_join }})",
        ),
        ("Destructure", "val ({{ vars|comma_join }}) = {{ cstruct }}"),
    ])
}
