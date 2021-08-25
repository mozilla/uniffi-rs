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

fn render_literal(oracle: &dyn CodeOracle, literal: &Literal) -> String {
    fn typed_number(oracle: &dyn CodeOracle, type_: &Type, num_str: String) -> String {
        match type_ {
            // special case Int32.
            Type::Int32 => num_str,
            // otherwise use constructor e.g. UInt8(x)
            Type::Int8
            | Type::UInt8
            | Type::Int16
            | Type::UInt16
            | Type::UInt32
            | Type::Int64
            | Type::UInt64
            | Type::Float32
            | Type::Float64 =>
            // XXX we should pass in the codetype itself.
            {
                format!("{}({})", oracle.find(type_).type_label(oracle), num_str)
            }
            _ => panic!("Unexpected literal: {} is not a number", num_str),
        }
    }

    match literal {
        Literal::Boolean(v) => format!("{}", v),
        Literal::String(s) => format!("\"{}\"", s),
        Literal::Int(i, radix, type_) => typed_number(
            oracle,
            type_,
            match radix {
                Radix::Octal => format!("0o{:o}", i),
                Radix::Decimal => format!("{}", i),
                Radix::Hexadecimal => format!("{:#x}", i),
            },
        ),
        Literal::UInt(i, radix, type_) => typed_number(
            oracle,
            type_,
            match radix {
                Radix::Octal => format!("0o{:o}", i),
                Radix::Decimal => format!("{}", i),
                Radix::Hexadecimal => format!("{:#x}", i),
            },
        ),
        Literal::Float(string, type_) => typed_number(oracle, type_, string.clone()),
        _ => unreachable!("Literal"),
    }
}

macro_rules! impl_code_type_for_primitive {
    ($T:ty, $class_name:literal, $helper_code:literal) => {
        paste! {
            #[derive(Template)]
            #[template(syntax = "swift", ext = "swift", escape = "none", source = $helper_code )]
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
                    format!("{}.read(from: {})", $class_name, nm)
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
    "Bool",
    r#"
    extension Bool: ViaFfi {
        fileprivate typealias FfiType = Int8

        fileprivate static func read(from buf: Reader) throws -> Bool {
            return try self.lift(buf.readInt())
        }

        fileprivate func write(into buf: Writer) {
            buf.writeInt(self.lower())
        }

        fileprivate static func lift(_ v: Int8) throws -> Bool {
            return v != 0
        }

        fileprivate func lower() -> Int8 {
            return self ? 1 : 0
        }
    }
"#
);

impl_code_type_for_primitive!(
    StringCodeType,
    "String",
    r#"
    extension String: ViaFfi {
        fileprivate typealias FfiType = RustBuffer

        fileprivate static func lift(_ v: FfiType) throws -> Self {
            defer {
                v.deallocate()
            }
            if v.data == nil {
                return String()
            }
            let bytes = UnsafeBufferPointer<UInt8>(start: v.data!, count: Int(v.len))
            return String(bytes: bytes, encoding: String.Encoding.utf8)!
        }

        fileprivate func lower() -> FfiType {
            return self.utf8CString.withUnsafeBufferPointer { ptr in
                // The swift string gives us int8_t, we want uint8_t.
                ptr.withMemoryRebound(to: UInt8.self) { ptr in
                    // The swift string gives us a trailing null byte, we don't want it.
                    let buf = UnsafeBufferPointer(rebasing: ptr.prefix(upTo: ptr.count - 1))
                    return RustBuffer.from(buf)
                }
            }
        }

        fileprivate static func read(from buf: Reader) throws -> Self {
            let len: Int32 = try buf.readInt()
            return String(bytes: try buf.readBytes(count: Int(len)), encoding: String.Encoding.utf8)!
        }

        fileprivate func write(into buf: Writer) {
            let len = Int32(self.utf8.count)
            buf.writeInt(len)
            buf.writeBytes(self.utf8)
        }
    }
    "#
);

impl_code_type_for_primitive!(
    Int8CodeType,
    "Int8",
    r#"
    extension Int8: Primitive, ViaFfi {
        fileprivate static func read(from buf: Reader) throws -> Int8 {
            return try self.lift(buf.readInt())
        }

        fileprivate func write(into buf: Writer) {
            buf.writeInt(self.lower())
        }
    }
"#
);

impl_code_type_for_primitive!(
    Int16CodeType,
    "Int16",
    r#"
    extension Int16: Primitive, ViaFfi {
        fileprivate static func read(from buf: Reader) throws -> Int16 {
            return try self.lift(buf.readInt())
        }

        fileprivate func write(into buf: Writer) {
            buf.writeInt(self.lower())
        }
    }
"#
);

impl_code_type_for_primitive!(
    Int32CodeType,
    "Int32",
    r#"
    extension Int32: Primitive, ViaFfi {
        fileprivate static func read(from buf: Reader) throws -> Int32 {
            return try self.lift(buf.readInt())
        }

        fileprivate func write(into buf: Writer) {
            buf.writeInt(self.lower())
        }
    }
"#
);

impl_code_type_for_primitive!(
    Int64CodeType,
    "Int64",
    r#"
    extension Int64: Primitive, ViaFfi {
        fileprivate static func read(from buf: Reader) throws -> Int64 {
            return try self.lift(buf.readInt())
        }

        fileprivate func write(into buf: Writer) {
            buf.writeInt(self.lower())
        }
    }
"#
);

impl_code_type_for_primitive!(
    UInt8CodeType,
    "UInt8",
    r#"
    extension UInt8: Primitive, ViaFfi {
        fileprivate static func read(from buf: Reader) throws -> UInt8 {
            return try self.lift(buf.readInt())
        }

        fileprivate func write(into buf: Writer) {
            buf.writeInt(self.lower())
        }
    }
"#
);

impl_code_type_for_primitive!(
    UInt16CodeType,
    "UInt16",
    r#"
    extension UInt16: Primitive, ViaFfi {
        fileprivate static func read(from buf: Reader) throws -> UInt16 {
            return try self.lift(buf.readInt())
        }

        fileprivate func write(into buf: Writer) {
            buf.writeInt(self.lower())
        }
    }

"#
);

impl_code_type_for_primitive!(
    UInt32CodeType,
    "UInt32",
    r#"
    extension UInt32: Primitive, ViaFfi {
        fileprivate static func read(from buf: Reader) throws -> UInt32 {
            return try self.lift(buf.readInt())
        }

        fileprivate func write(into buf: Writer) {
            buf.writeInt(self.lower())
        }
    }
"#
);

impl_code_type_for_primitive!(
    UInt64CodeType,
    "UInt64",
    r#"
    extension UInt64: Primitive, ViaFfi {
        fileprivate static func read(from buf: Reader) throws -> UInt64 {
            return try self.lift(buf.readInt())
        }

        fileprivate func write(into buf: Writer) {
            buf.writeInt(self.lower())
        }
    }
"#
);

impl_code_type_for_primitive!(
    Float32CodeType,
    "Float",
    r#"
    extension Float: Primitive, ViaFfi {
        fileprivate static func read(from buf: Reader) throws -> Float {
            return try self.lift(buf.readFloat())
        }

        fileprivate func write(into buf: Writer) {
            buf.writeFloat(self.lower())
        }
    }
"#
);

impl_code_type_for_primitive!(
    Float64CodeType,
    "Double",
    r#"
    extension Double: Primitive, ViaFfi {
        fileprivate static func read(from buf: Reader) throws -> Double {
            return try self.lift(buf.readDouble())
        }

        fileprivate func write(into buf: Writer) {
            buf.writeDouble(self.lower())
        }
    }
"#
);
