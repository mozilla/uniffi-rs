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
    object FFIConverterBoolean {
        internal fun lift(v: Byte): Boolean {
            return v.toInt() != 0
        }

        internal fun read(buf: ByteBuffer): Boolean {
            return lift(buf.get())
        }

        internal fun lower(v: Boolean): Byte {
            return if (v) 1.toByte() else 0.toByte()
        }

        internal fun write(v: Boolean, buf: RustBufferBuilder) {
            buf.putByte(lower(v))
        }
    }
"#
);

impl_code_type_for_primitive!(
    StringCodeType,
    "String",
    r#"
    object FFIConverterString {
        internal fun lift(rbuf: RustBuffer.ByValue): String {
            try {
                val byteArr = ByteArray(rbuf.len)
                rbuf.asByteBuffer()!!.get(byteArr)
                return byteArr.toString(Charsets.UTF_8)
            } finally {
                RustBuffer.free(rbuf)
            }
        }

        internal fun read(buf: ByteBuffer): String {
            val len = buf.getInt()
            val byteArr = ByteArray(len)
            buf.get(byteArr)
            return byteArr.toString(Charsets.UTF_8)
        }

        internal fun lower(v: String): RustBuffer.ByValue {
            val byteArr = v.toByteArray(Charsets.UTF_8)
            // Ideally we'd pass these bytes to `ffi_bytebuffer_from_bytes`, but doing so would require us
            // to copy them into a JNA `Memory`. So we might as well directly copy them into a `RustBuffer`.
            val rbuf = RustBuffer.alloc(byteArr.size)
            rbuf.asByteBuffer()!!.put(byteArr)
            return rbuf
        }

        internal fun write(v: String, buf: RustBufferBuilder) {
            val byteArr = v.toByteArray(Charsets.UTF_8)
            buf.putInt(byteArr.size)
            buf.put(byteArr)
        }
    }
    "#
);

impl_code_type_for_primitive!(
    Int8CodeType,
    "Byte",
    r#"
    object FFIConverterByte {
        internal fun lift(v: Byte): Byte {
            return v
        }

        internal fun read(buf: ByteBuffer): Byte {
            return buf.get()
        }

        internal fun lower(v: Byte): Byte {
            return v
        }

        internal fun write(v: Byte, buf: RustBufferBuilder) {
            buf.putByte(v)
        }
    }
"#
);

impl_code_type_for_primitive!(
    Int16CodeType,
    "Short",
    r#"
    object FFIConverterShort {
        internal fun lift(v: Short): Short {
            return v
        }

        internal fun read(buf: ByteBuffer): Short {
            return buf.getShort()
        }

        internal fun lower(v: Short): Short {
            return v
        }

        internal fun write(v: Short, buf: RustBufferBuilder) {
            buf.putShort(v)
        }
    }
"#
);

impl_code_type_for_primitive!(
    Int32CodeType,
    "Int",
    r#"
    object FFIConverterInt {
        internal fun lift(v: Int): Int {
            return v
        }

        internal fun read(buf: ByteBuffer): Int {
            return buf.getInt()
        }

        internal fun lower(v: Int): Int {
            return v
        }

        internal fun write(v: Int, buf: RustBufferBuilder) {
            buf.putInt(v)
        }
    }
"#
);

impl_code_type_for_primitive!(
    Int64CodeType,
    "Long",
    r#"
    object FFIConverterLong {
        internal fun lift(v: Long): Long {
            return v
        }

        internal fun read(buf: ByteBuffer): Long {
            return buf.getLong()
        }

        internal fun lower(v: Long): Long {
            return v
        }

        internal fun write(v: Long, buf: RustBufferBuilder) {
            buf.putLong(v)
        }
    }
"#
);

impl_code_type_for_primitive!(
    UInt8CodeType,
    "UByte",
    r#"
    @ExperimentalUnsignedTypes
    object FFIConverterUByte {
        internal fun lift(v: Byte): UByte {
            return v.toUByte()
        }

        internal fun read(buf: ByteBuffer): UByte {
            return lift(buf.get())
        }

        internal fun lower(v: UByte): Byte {
            return v.toByte()
        }

        internal fun write(v: UByte, buf: RustBufferBuilder) {
            buf.putByte(v.toByte())
        }
    }
"#
);

impl_code_type_for_primitive!(
    UInt16CodeType,
    "UShort",
    r#"
    @ExperimentalUnsignedTypes
    object FFIConverterUShort {
        internal fun lift(v: Short): UShort {
            return v.toUShort()
        }

        internal fun read(buf: ByteBuffer): UShort {
            return lift(buf.getShort())
        }

        internal fun lower(v: UShort): Short {
            return v.toShort()
        }

        internal fun write(v: UShort, buf: RustBufferBuilder) {
            buf.putShort(v.toShort())
        }
    }
"#
);

impl_code_type_for_primitive!(
    UInt32CodeType,
    "UInt",
    r#"
    @ExperimentalUnsignedTypes
    object FFIConverterUInt {
        internal fun lift(v: Int): UInt {
            return v.toUInt()
        }

        internal fun read(buf: ByteBuffer): UInt {
            return lift(buf.getInt())
        }

        internal fun lower(v: UInt): Int {
            return v.toInt()
        }

        internal fun write(v: UInt, buf: RustBufferBuilder) {
            buf.putInt(v.toInt())
        }
    }
"#
);

impl_code_type_for_primitive!(
    UInt64CodeType,
    "ULong",
    r#"
    @ExperimentalUnsignedTypes
    object FFIConverterULong {
        internal fun lift(v: Long): ULong {
            return v.toULong()
        }

        internal fun read(buf: ByteBuffer): ULong {
            return lift(buf.getLong())
        }

        internal fun lower(v: ULong): Long {
            return v.toLong()
        }

        internal fun write(v: ULong, buf: RustBufferBuilder) {
            buf.putLong(v.toLong())
        }
    }
"#
);

impl_code_type_for_primitive!(
    Float32CodeType,
    "Float",
    r#"
    object FFIConverterFloat {
        internal fun lift(v: Float): Float {
            return v
        }

        internal fun read(buf: ByteBuffer): Float {
            return buf.getFloat()
        }

        internal fun lower(v: Float): Float {
            return v
        }

        internal fun write(v: Float, buf: RustBufferBuilder) {
            buf.putFloat(v)
        }
    }
"#
);

impl_code_type_for_primitive!(
    Float64CodeType,
    "Double",
    r#"
    object FFIConverterDouble {
        internal fun lift(v: Double): Double {
            return v
        }

        internal fun read(buf: ByteBuffer): Double {
            val v = buf.getDouble()
            return v
        }

        internal fun lower(v: Double): Double {
            return v
        }

        internal fun write(v: Double, buf: RustBufferBuilder) {
            buf.putDouble(v)
        }
    }
"#
);
