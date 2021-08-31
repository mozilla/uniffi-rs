/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::bindings::backend::{CodeOracle, CodeType, Literal};
use crate::interface::{types::Type, Radix};
use askama::Template;
use paste::paste;

#[allow(unused_imports)]
use super::filters;

fn render_literal(_oracle: &dyn CodeOracle, literal: &Literal) -> String {
    fn typed_number(type_: &Type, num_str: String) -> String {
        match type_ {
            // Bytes, Shorts and Ints can all be inferred from the type.
            Type::Int8 | Type::Int16 | Type::Int32 => num_str,
            Type::Int64 => format!("{}L", num_str),

            Type::UInt8 | Type::UInt16 | Type::UInt32 => format!("{}u", num_str),
            Type::UInt64 => format!("{}uL", num_str),

            Type::Float32 => format!("{}f", num_str),
            Type::Float64 => num_str,
            _ => panic!("Unexpected literal: {} is not a number", num_str),
        }
    }

    match literal {
        Literal::Boolean(v) => format!("{}", v),
        Literal::String(s) => format!("\"{}\"", s),
        Literal::Int(i, radix, type_) => typed_number(
            &type_,
            match radix {
                Radix::Octal => format!("{:#x}", i),
                Radix::Decimal => format!("{}", i),
                Radix::Hexadecimal => format!("{:#x}", i),
            },
        ),
        Literal::UInt(i, radix, type_) => typed_number(
            &type_,
            match radix {
                Radix::Octal => format!("{:#x}", i),
                Radix::Decimal => format!("{}", i),
                Radix::Hexadecimal => format!("{:#x}", i),
            },
        ),
        Literal::Float(string, type_) => typed_number(&type_, string.clone()),

        _ => unreachable!("Literal"),
    }
}

macro_rules! impl_code_type_for_primitive {
    ($T:ty, $class_name:literal, $helper_code:literal) => {
        paste! {
            #[derive(Template)]
            #[template(syntax = "kt", ext = "kt", escape = "none", source = $helper_code )]
            pub struct $T;

            impl CodeType for $T  {
                fn type_label(&self, _oracle: &dyn CodeOracle) -> String {
                    $class_name.into()
                }

                fn literal(&self, oracle: &dyn CodeOracle, literal: &Literal) -> String {
                    render_literal(oracle, &literal)
                }

                fn helper_code(&self, _oracle: &dyn CodeOracle) -> Option<String> {
                    Some(self.render().unwrap())
                }
            }
        }
    }
}

impl_code_type_for_primitive!(
    BooleanCodeType,
    "Boolean",
    r#"
    object FFIConverterBoolean: FFIConverter<Boolean, Byte> {
        override fun lift(v: Byte): Boolean {
            return v.toInt() != 0
        }

        override fun read(buf: ByteBuffer): Boolean {
            return lift(buf.get())
        }

        override fun lower(v: Boolean): Byte {
            return if (v) 1.toByte() else 0.toByte()
        }

        override fun write(v: Boolean, bufferWrite: BufferWriteFunc) {
            putByte(lower(v), bufferWrite)
        }
    }
"#
);

impl_code_type_for_primitive!(
    StringCodeType,
    "String",
    r#"
    object FFIConverterString: FFIConverter<String, RustBuffer.ByValue> {
        override fun lift(v: RustBuffer.ByValue): String {
            try {
                val byteArr = ByteArray(v.len)
                v.asByteBuffer()!!.get(byteArr)
                return byteArr.toString(Charsets.UTF_8)
            } finally {
                RustBuffer.free(v)
            }
        }

        override fun read(buf: ByteBuffer): String {
            val len = buf.getInt()
            val byteArr = ByteArray(len)
            buf.get(byteArr)
            return byteArr.toString(Charsets.UTF_8)
        }

        override fun lower(v: String): RustBuffer.ByValue {
            val byteArr = v.toByteArray(Charsets.UTF_8)
            // Ideally we'd pass these bytes to `ffi_bytebuffer_from_bytes`, but doing so would require us
            // to copy them into a JNA `Memory`. So we might as well directly copy them into a `RustBuffer`.
            val rbuf = RustBuffer.alloc(byteArr.size)
            rbuf.asByteBuffer()!!.put(byteArr)
            return rbuf
        }

        override fun write(v: String, bufferWrite: BufferWriteFunc) {
            val byteArr = v.toByteArray(Charsets.UTF_8)
            putInt(byteArr.size, bufferWrite)
            put(byteArr, bufferWrite)
        }
    }
    "#
);

impl_code_type_for_primitive!(
    Int8CodeType,
    "Byte",
    r#"
    object FFIConverterByte: FFIConverter<Byte, Byte> {
        override fun lift(v: Byte): Byte {
            return v
        }

        override fun read(buf: ByteBuffer): Byte {
            return buf.get()
        }

        override fun lower(v: Byte): Byte {
            return v
        }

        override fun write(v: Byte, bufferWrite: BufferWriteFunc) {
            putByte(v, bufferWrite)
        }
    }
"#
);

impl_code_type_for_primitive!(
    Int16CodeType,
    "Short",
    r#"
    object FFIConverterShort: FFIConverter<Short, Short> {
        override fun lift(v: Short): Short {
            return v
        }

        override fun read(buf: ByteBuffer): Short {
            return buf.getShort()
        }

        override fun lower(v: Short): Short {
            return v
        }

        override fun write(v: Short, bufferWrite: BufferWriteFunc) {
            putShort(v, bufferWrite)
        }
    }
"#
);

impl_code_type_for_primitive!(
    Int32CodeType,
    "Int",
    r#"
    object FFIConverterInt: FFIConverter<Int, Int> {
        override fun lift(v: Int): Int {
            return v
        }

        override fun read(buf: ByteBuffer): Int {
            return buf.getInt()
        }

        override fun lower(v: Int): Int {
            return v
        }

        override fun write(v: Int, bufferWrite: BufferWriteFunc) {
            putInt(v, bufferWrite)
        }
    }
"#
);

impl_code_type_for_primitive!(
    Int64CodeType,
    "Long",
    r#"
    object FFIConverterLong: FFIConverter<Long, Long> {
        override fun lift(v: Long): Long {
            return v
        }

        override fun read(buf: ByteBuffer): Long {
            return buf.getLong()
        }

        override fun lower(v: Long): Long {
            return v
        }

        override fun write(v: Long, bufferWrite: BufferWriteFunc) {
            putLong(v, bufferWrite)
        }
    }
"#
);

impl_code_type_for_primitive!(
    UInt8CodeType,
    "UByte",
    r#"
    @ExperimentalUnsignedTypes
    object FFIConverterUByte: FFIConverter<UByte, Byte> {
        override fun lift(v: Byte): UByte {
            return v.toUByte()
        }

        override fun read(buf: ByteBuffer): UByte {
            return lift(buf.get())
        }

        override fun lower(v: UByte): Byte {
            return v.toByte()
        }

        override fun write(v: UByte, bufferWrite: BufferWriteFunc) {
            putByte(v.toByte(), bufferWrite)
        }
    }
"#
);

impl_code_type_for_primitive!(
    UInt16CodeType,
    "UShort",
    r#"
    @ExperimentalUnsignedTypes
    object FFIConverterUShort: FFIConverter<UShort, Short> {
        override fun lift(v: Short): UShort {
            return v.toUShort()
        }

        override fun read(buf: ByteBuffer): UShort {
            return lift(buf.getShort())
        }

        override fun lower(v: UShort): Short {
            return v.toShort()
        }

        override fun write(v: UShort, bufferWrite: BufferWriteFunc) {
            putShort(v.toShort(), bufferWrite)
        }
    }
"#
);

impl_code_type_for_primitive!(
    UInt32CodeType,
    "UInt",
    r#"
    @ExperimentalUnsignedTypes
    object FFIConverterUInt: FFIConverter<UInt, Int> {
        override fun lift(v: Int): UInt {
            return v.toUInt()
        }

        override fun read(buf: ByteBuffer): UInt {
            return lift(buf.getInt())
        }

        override fun lower(v: UInt): Int {
            return v.toInt()
        }

        override fun write(v: UInt, bufferWrite: BufferWriteFunc) {
            putInt(v.toInt(), bufferWrite)
        }
    }
"#
);

impl_code_type_for_primitive!(
    UInt64CodeType,
    "ULong",
    r#"
    @ExperimentalUnsignedTypes
    object FFIConverterULong: FFIConverter<ULong, Long> {
        override fun lift(v: Long): ULong {
            return v.toULong()
        }

        override fun read(buf: ByteBuffer): ULong {
            return lift(buf.getLong())
        }

        override fun lower(v: ULong): Long {
            return v.toLong()
        }

        override fun write(v: ULong, bufferWrite: BufferWriteFunc) {
            putLong(v.toLong(), bufferWrite)
        }
    }
"#
);

impl_code_type_for_primitive!(
    Float32CodeType,
    "Float",
    r#"
    object FFIConverterFloat: FFIConverter<Float, Float> {
        override fun lift(v: Float): Float {
            return v
        }

        override fun read(buf: ByteBuffer): Float {
            return buf.getFloat()
        }

        override fun lower(v: Float): Float {
            return v
        }

        override fun write(v: Float, bufferWrite: BufferWriteFunc) {
            putFloat(v, bufferWrite)
        }
    }
"#
);

impl_code_type_for_primitive!(
    Float64CodeType,
    "Double",
    r#"
    object FFIConverterDouble: FFIConverter<Double, Double> {
        override fun lift(v: Double): Double {
            return v
        }

        override fun read(buf: ByteBuffer): Double {
            val v = buf.getDouble()
            return v
        }

        override fun lower(v: Double): Double {
            return v
        }

        override fun write(v: Double, bufferWrite: BufferWriteFunc) {
            putDouble(v, bufferWrite)
        }
    }
"#
);
