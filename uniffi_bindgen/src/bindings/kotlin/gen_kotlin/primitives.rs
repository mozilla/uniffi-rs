/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::bindings::backend::{CodeOracle, CodeType, Literal};
use crate::interface::{types::Type, Radix};
use askama::Template;
use paste::paste;
use std::fmt;

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

                fn lower(&self, oracle: &dyn CodeOracle, nm: &dyn fmt::Display) -> String {
                    format!("{}.lower()", oracle.var_name(nm))
                }

                fn write(&self, oracle: &dyn CodeOracle, nm: &dyn fmt::Display, target: &dyn fmt::Display) -> String {
                    format!("{}.write({})", oracle.var_name(nm), target)
                }

                fn lift(&self, _oracle: &dyn CodeOracle, nm: &dyn fmt::Display) -> String {
                    format!("{}.lift({})", $class_name, nm)
                }

                fn read(&self, _oracle: &dyn CodeOracle, nm: &dyn fmt::Display) -> String {
                    format!("{}.read({})", $class_name, nm)
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
    internal fun Boolean.Companion.lift(v: Byte): Boolean {
        return v.toInt() != 0
    }

    internal fun Boolean.Companion.read(buf: ByteBuffer): Boolean {
        return Boolean.lift(buf.get())
    }

    internal fun Boolean.lower(): Byte {
        return if (this) 1.toByte() else 0.toByte()
    }

    internal fun Boolean.write(buf: RustBufferBuilder) {
        buf.putByte(this.lower())
    }
"#
);

impl_code_type_for_primitive!(
    StringCodeType,
    "String",
    r#"
    internal fun String.Companion.lift(rbuf: RustBuffer.ByValue): String {
        try {
            val byteArr = ByteArray(rbuf.len)
            rbuf.asByteBuffer()!!.get(byteArr)
            return byteArr.toString(Charsets.UTF_8)
        } finally {
            RustBuffer.free(rbuf)
        }
    }

    internal fun String.Companion.read(buf: ByteBuffer): String {
        val len = buf.getInt()
        val byteArr = ByteArray(len)
        buf.get(byteArr)
        return byteArr.toString(Charsets.UTF_8)
    }

    internal fun String.lower(): RustBuffer.ByValue {
        val byteArr = this.toByteArray(Charsets.UTF_8)
        // Ideally we'd pass these bytes to `ffi_bytebuffer_from_bytes`, but doing so would require us
        // to copy them into a JNA `Memory`. So we might as well directly copy them into a `RustBuffer`.
        val rbuf = RustBuffer.alloc(byteArr.size)
        rbuf.asByteBuffer()!!.put(byteArr)
        return rbuf
    }

    internal fun String.write(buf: RustBufferBuilder) {
        val byteArr = this.toByteArray(Charsets.UTF_8)
        buf.putInt(byteArr.size)
        buf.put(byteArr)
    }
    "#
);

impl_code_type_for_primitive!(
    Int8CodeType,
    "Byte",
    r#"
    internal fun Byte.Companion.lift(v: Byte): Byte {
        return v
    }

    internal fun Byte.Companion.read(buf: ByteBuffer): Byte {
        return buf.get()
    }

    internal fun Byte.lower(): Byte {
        return this
    }

    internal fun Byte.write(buf: RustBufferBuilder) {
        buf.putByte(this)
    }
"#
);

impl_code_type_for_primitive!(
    Int16CodeType,
    "Short",
    r#"
    internal fun Short.Companion.lift(v: Short): Short {
        return v
    }

    internal fun Short.Companion.read(buf: ByteBuffer): Short {
        return buf.getShort()
    }

    internal fun Short.lower(): Short {
        return this
    }

    internal fun Short.write(buf: RustBufferBuilder) {
        buf.putShort(this)
    }
"#
);

impl_code_type_for_primitive!(
    Int32CodeType,
    "Int",
    r#"
    internal fun Int.Companion.lift(v: Int): Int {
        return v
    }

    internal fun Int.Companion.read(buf: ByteBuffer): Int {
        return buf.getInt()
    }

    internal fun Int.lower(): Int {
        return this
    }

    internal fun Int.write(buf: RustBufferBuilder) {
        buf.putInt(this)
    }
"#
);

impl_code_type_for_primitive!(
    Int64CodeType,
    "Long",
    r#"
    internal fun Long.Companion.lift(v: Long): Long {
        return v
    }

    internal fun Long.Companion.read(buf: ByteBuffer): Long {
        return buf.getLong()
    }

    internal fun Long.lower(): Long {
        return this
    }

    internal fun Long.write(buf: RustBufferBuilder) {
        buf.putLong(this)
    }
"#
);

impl_code_type_for_primitive!(
    UInt8CodeType,
    "UByte",
    r#"
    @ExperimentalUnsignedTypes
    internal fun UByte.Companion.lift(v: Byte): UByte {
        return v.toUByte()
    }

    @ExperimentalUnsignedTypes
    internal fun UByte.Companion.read(buf: ByteBuffer): UByte {
        return UByte.lift(buf.get())
    }

    @ExperimentalUnsignedTypes
    internal fun UByte.lower(): Byte {
        return this.toByte()
    }

    @ExperimentalUnsignedTypes
    internal fun UByte.write(buf: RustBufferBuilder) {
        buf.putByte(this.toByte())
    }
"#
);

impl_code_type_for_primitive!(
    UInt16CodeType,
    "UShort",
    r#"
    @ExperimentalUnsignedTypes
    internal fun UShort.Companion.lift(v: Short): UShort {
        return v.toUShort()
    }

    @ExperimentalUnsignedTypes
    internal fun UShort.Companion.read(buf: ByteBuffer): UShort {
        return UShort.lift(buf.getShort())
    }

    @ExperimentalUnsignedTypes
    internal fun UShort.lower(): Short {
        return this.toShort()
    }

    @ExperimentalUnsignedTypes
    internal fun UShort.write(buf: RustBufferBuilder) {
        buf.putShort(this.toShort())
    }
"#
);

impl_code_type_for_primitive!(
    UInt32CodeType,
    "UInt",
    r#"
    @ExperimentalUnsignedTypes
    internal fun UInt.Companion.lift(v: Int): UInt {
        return v.toUInt()
    }

    @ExperimentalUnsignedTypes
    internal fun UInt.Companion.read(buf: ByteBuffer): UInt {
        return UInt.lift(buf.getInt())
    }

    @ExperimentalUnsignedTypes
    internal fun UInt.lower(): Int {
        return this.toInt()
    }

    @ExperimentalUnsignedTypes
    internal fun UInt.write(buf: RustBufferBuilder) {
        buf.putInt(this.toInt())
    }
"#
);

impl_code_type_for_primitive!(
    UInt64CodeType,
    "ULong",
    r#"
    @ExperimentalUnsignedTypes
    internal fun ULong.Companion.lift(v: Long): ULong {
        return v.toULong()
    }

    @ExperimentalUnsignedTypes
    internal fun ULong.Companion.read(buf: ByteBuffer): ULong {
        return ULong.lift(buf.getLong())
    }

    @ExperimentalUnsignedTypes
    internal fun ULong.lower(): Long {
        return this.toLong()
    }

    @ExperimentalUnsignedTypes
    internal fun ULong.write(buf: RustBufferBuilder) {
        buf.putLong(this.toLong())
    }
"#
);

impl_code_type_for_primitive!(
    Float32CodeType,
    "Float",
    r#"
    internal fun Float.Companion.lift(v: Float): Float {
        return v
    }

    internal fun Float.Companion.read(buf: ByteBuffer): Float {
        return buf.getFloat()
    }

    internal fun Float.lower(): Float {
        return this
    }

    internal fun Float.write(buf: RustBufferBuilder) {
        buf.putFloat(this)
    }
"#
);

impl_code_type_for_primitive!(
    Float64CodeType,
    "Double",
    r#"
    internal fun Double.Companion.lift(v: Double): Double {
        return v
    }

    internal fun Double.Companion.read(buf: ByteBuffer): Double {
        val v = buf.getDouble()
        return v
    }

    internal fun Double.lower(): Double {
        return this
    }

    internal fun Double.write(buf: RustBufferBuilder) {
        buf.putDouble(this)
    }
"#
);
