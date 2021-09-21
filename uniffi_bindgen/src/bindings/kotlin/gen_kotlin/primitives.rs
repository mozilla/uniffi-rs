/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::{names, CodeBuilder, KotlinCodeName, KotlinCodeType};
use crate::codegen::{NewCodeType, PrimitiveTypeHandler};
use crate::interface::{ComponentInterface, Literal, Radix, Type};
use askama::Template;

impl KotlinCodeType for PrimitiveTypeHandler {
    fn nm(&self) -> String {
        match self {
            Self::UInt8 => "UByte",
            Self::Int8 => "Byte",
            Self::UInt16 => "UShort",
            Self::Int16 => "Short",
            Self::UInt32 => "UInt",
            Self::Int32 => "Int",
            Self::UInt64 => "ULong",
            Self::Int64 => "Long",
            Self::Float32 => "Float",
            Self::Float64 => "Double",
            Self::Boolean => "Boolean",
            Self::String => "String",
        }
        .into()
    }

    fn declare_code(&self, code_builder: CodeBuilder, _ci: &ComponentInterface) -> CodeBuilder {
        // Needs to be separated out because String uses a different template struct than the rest
        if matches!(self, Self::String) {
            return code_builder.code_block(StringTemplate);
        }

        code_builder.code_block(match self {
            Self::Boolean => PrimitiveTemplate::boolean(self),
            Self::UInt8 => PrimitiveTemplate::uint(self, "get"),
            Self::UInt16 => PrimitiveTemplate::uint(self, "getShort"),
            Self::UInt32 => PrimitiveTemplate::uint(self, "getInt"),
            Self::UInt64 => PrimitiveTemplate::uint(self, "getLong"),
            Self::Int8 => PrimitiveTemplate::int(self, "get"),
            Self::Int16 => PrimitiveTemplate::int(self, "getShort"),
            Self::Int32 => PrimitiveTemplate::int(self, "getInt"),
            Self::Int64 => PrimitiveTemplate::int(self, "getLong"),
            Self::Float32 => PrimitiveTemplate::int(self, "getFloat"),
            Self::Float64 => PrimitiveTemplate::int(self, "getDouble"),
            Self::String => unreachable!(),
        })
    }

    fn lower(&self, nm: &str) -> String {
        format!("{}.lower()", names::var_name(nm))
    }

    fn write(&self, nm: &str, target: &str) -> String {
        format!("{}.write({})", names::var_name(nm), target)
    }

    fn lift(&self, nm: &str) -> String {
        format!("{}.lift({})", self.nm(), nm)
    }

    fn read(&self, nm: &str) -> String {
        format!("{}.read({})", self.nm(), nm)
    }

    fn literal(&self, literal: &Literal) -> String {
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
}

#[derive(Template, Hash)]
#[template(syntax = "kt", escape = "none", path = "String.kt")]
struct StringTemplate;

#[derive(Template, Hash)]
#[template(syntax = "kt", escape = "none", path = "Primitive.kt")]
struct PrimitiveTemplate {
    type_name: String,
    ffi_name: String,
    lift_expr: String,
    lower_expr: String,
    read_expr: String,
    write_expr: String,
}

impl PrimitiveTemplate {
    fn int(handler: &PrimitiveTypeHandler, get_func: &str) -> Self {
        Self {
            type_name: handler.nm(),
            ffi_name: handler.ffi_type().nm(),
            lift_expr: "v".into(),
            read_expr: format!("buf.{}()", get_func),
            lower_expr: "this".into(),
            write_expr: format!("buf.put{}(this)", handler.ffi_type().nm()),
        }
    }

    fn uint(handler: &PrimitiveTypeHandler, get_func: &str) -> Self {
        let type_name = handler.nm();
        let ffi_name = handler.ffi_type().nm();
        Self {
            type_name: handler.nm(),
            ffi_name: handler.ffi_type().nm(),
            lift_expr: format!("v.to{}()", type_name),
            read_expr: format!("{}.lift(buf.{}())", type_name, get_func),
            lower_expr: format!("this.to{}()", ffi_name),
            write_expr: format!("buf.put{}(this.to{}())", ffi_name, ffi_name),
        }
    }

    fn boolean(handler: &PrimitiveTypeHandler) -> Self {
        Self {
            type_name: handler.nm(),
            ffi_name: handler.ffi_type().nm(),
            lift_expr: "v.toInt() != 0".into(),
            read_expr: "Boolean.lift(buf.get())".into(),
            lower_expr: "if (this) 1.toByte() else 0.toByte()".into(),
            write_expr: "buf.putByte(this.lower())".into(),
        }
    }
}
