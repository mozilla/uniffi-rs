use bindings_ir::{Renderer, ir::Type};
use tera::{try_get_value, Result, Value};

pub fn setup_renderer(renderer: &mut Renderer) -> Result<()> {
    renderer
        .tera_mut()
        .register_filter("quote_string", |value: &'_ Value, _: &_| {
            let value = try_get_value!("quote_string", "value", String, value);
            // quote all '$' chars
            Ok(Value::String(format!("\"{}\"", value.replace('$', "\\$"))))
        });
    renderer.tera_mut()
        .register_filter("cast_int_fn", |value: &'_ Value, _: &_| {
            let value = try_get_value!("cast_int", "type", Type, value);
            Ok(match value {
                Type::Int8 => "toByte",
                Type::Int16 => "toShort",
                Type::Int32 => "toInt",
                Type::Int64 => "toLong",
                Type::UInt8 => "toUByte",
                Type::UInt16 => "toUShort",
                Type::UInt32 => "toUInt",
                Type::UInt64 => "toULong",
                _ => unimplemented!(),
            }.into())
        });
    renderer.add_ast_templates([
        ("String", "String"),
        ("Boolean", "Boolean"),
        ("Object", "{{ class }}"),
        ("Reference", "{{ inner }}"),
        ("List", "MutableList<{{ inner }}>"),
        ("Map", "MutableMap<{{ key }}, {{ value }}>"),
        ("Nullable", "{{ inner }}?"),
        ("UInt8", "UByte"),
        ("Int8", "Byte"),
        ("UInt16", "UShort"),
        ("Int16", "Short"),
        ("UInt32", "UInt"),
        ("Int32", "Int"),
        ("UInt64", "ULong"),
        ("Int64", "Long"),
        ("Float32", "Float"),
        ("Float64", "Double"),
        ("Pointer", "com.sun.jna.Pointer"),
        ("CStruct", "{{ name }}"),
        ("LiteralBoolean", "{{ value }}"),
        ("LiteralString", "{{ value|quote_string }}"),
        ("LiteralNull", "null"),
        ("LiteralInt", "{{ value }}"),
        ("LiteralInt8", "({{ value }}).toByte()"),
        ("LiteralInt16", "({{ value }}).toShort()"),
        ("LiteralInt32", "{{ value }}"),
        ("LiteralInt64", "{{ value }}L"),
        ("LiteralUInt8", "({{ value }}).toUByte()"),
        ("LiteralUInt16", "({{ value }}).toUShort()"),
        // Need to use L suffix for U32 literals to ensure enough width
        ("LiteralUInt32", "({{ value }}L).toUInt()"),
        ("LiteralUInt64", "({{ value }}L).toULong()"),
        ("LiteralFloat32", "({{ value }}).toFloat()"),
        ("LiteralFloat64", "({{ value }}).toDouble()"),
        ("CastInt8", "({{ value }}).toByte()"),
        ("CastInt16", "({{ value }}).toShort()"),
        ("CastInt32", "{{ value }}.toInt()"),
        ("CastInt64", "{{ value }}.toLong()"),
        ("CastUInt8", "({{ value }}).toUByte()"),
        ("CastUInt16", "({{ value }}).toUShort()"),
        ("CastUInt32", "({{ value }}).toUInt()"),
        ("CastUInt64", "({{ value }}).toULong()"),
    ])
}
