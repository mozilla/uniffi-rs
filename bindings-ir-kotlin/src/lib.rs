/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::collections::HashSet;
use tera::Result;

mod buffer;
mod class;
mod collections;
mod ffi;
#[cfg(test)]
mod tests;
mod types;

use bindings_ir::{ir::*, Renderer, TeraArgs};
use heck::ToUpperCamelCase;
use tera::{to_value, try_get_value, Value};

pub fn render_module(module: Module) -> Result<String> {
    render_module_with_mode(module, RenderMode::Normal)
}

pub fn render_test_script(module: Module) -> Result<String> {
    render_module_with_mode(module, RenderMode::TestScript)
}

pub fn render_module_with_mode(module: Module, mode: RenderMode) -> Result<String> {
    let mut renderer = Renderer::new();
    renderer.add_ast_templates([
        ("Module", include_str!("templates/Module.kt")),
        ("NativeLibrary", include_str!("templates/NativeLibrary.kt")),
        ("FunctionDef", include_str!("templates/FunctionDef.kt")),
        ("Block", include_str!("templates/Block.kt")),
        // Statements
        ("ExpressionStatement", "{{ expr }}"),
        ("Return", "return{% if value %} {{ value }}{% endif %}"),
        ("Val", "val {{ name }}: {{ type }} = {{ initial }}"),
        ("Var", "var {{ name }}: {{ type }} = {{ initial }}"),
        ("Assign", "{{ name }} = {{ value }}"),
        ("Set", "{{ container }}.{{ field }} = {{ value }}"),
        ("Assert", "assert({{ value }})"),
        ("AssertRaises", include_str!("templates/AssertRaises.kt")),
        (
            "AssertRaisesWithString",
            include_str!("templates/AssertRaisesWithString.kt"),
        ),
        ("Gc", include_str!("templates/Gc.kt")),
        ("If", include_str!("templates/If.kt")),
        ("For", include_str!("templates/For.kt")),
        ("Loop", include_str!("templates/Loop.kt")),
        ("Break", "break"),
        ("MatchEnum", include_str!("templates/MatchEnum.kt")),
        ("MatchInt", include_str!("templates/MatchInt.kt")),
        ("MatchNullable", include_str!("templates/MatchNullable.kt")),
        // Expressions
        ("Ident", "{{ name }}"),
        ("Get", "{{ container }}.{{ field }}"),
        ("This", "this"),
        ("Call", "{{ name }}({{ values|comma_join }})"),
        ("IsInstance", "({{ value }} is {{ class }})"),
        ("Eq", "({{ first }} == {{ second }})"),
        ("Gt", "({{ first }} > {{ second }})"),
        ("Ge", "({{ first }} >= {{ second }})"),
        ("Lt", "({{ first }} < {{ second }})"),
        ("Le", "({{ first }} <= {{ second }})"),
        ("Add", "({{ first }} + {{ second }}).{{ type|cast_int_fn }}()"),
        ("Sub", "({{ first }} - {{ second }}).{{ type|cast_int_fn }}()"),
        ("Mul", "({{ first }} * {{ second }}).{{ type|cast_int_fn }}()"),
        ("Div", "({{ first }} / {{ second }}).{{ type|cast_int_fn }}()"),
        ("And", "({{ first }} && {{ second }})"),
        ("Or", "({{ first }} || {{ second }})"),
        ("Not", "(!{{ value }})"),
        ("StrMinByteLen", "({{ string }}.length * 3)"),
        ("StrConcat", include_str!("templates/StrConcat.kt")),
        ("Raise", "throw {{ exception }}"),
        (
            "RaiseInternalException",
            "throw InternalException({{ message }})",
        ),
        // Kotlin doesn't differentiate between nullable values vs non-nullable
        ("Some", "{{ inner }}"),
        (
            "Unwrap",
            "({{ nullable }} ?: throw RuntimeException({{ message }}))",
        ),
        // Kotlin always uses references, so nothing is needed for the Ref expression
        ("Ref", "{{ name }}"),
        ("Argument", "{{ name }}: {{ type }}"),
        (
            "Field",
            "{% if mutable %}var{% else %}val{% endif %} {{ name }}: {{ type }}",
        ),
        // Names
        ("ClassName", "{{ name|to_class_name }}"),
        ("CStructName", "{{ name|to_upper_camel_case }}"),
        ("ArgName", "{{ name|to_lower_camel_case }}"),
        ("FieldName", "{{ name|to_lower_camel_case }}"),
        ("FunctionName", "{{ name|to_lower_camel_case }}"),
        ("VarName", "{{ name|to_lower_camel_case }}"),
    ])?;
    match mode {
        RenderMode::Normal => {
            renderer.add_ast_templates([("Private", "internal"), ("Public", "public")])?;
        }
        RenderMode::TestScript => {
            // Ignore visibility modifiers when rendering test scripts.  They won't have any effect
            // and it's invalid to set a visibility modifier for a function in a test script, since
            // they're technically local functions.
            renderer.add_ast_templates([("Private", ""), ("Public", "")])?;
        }
    }
    let exception_names: HashSet<String> = module
        .iter_definitions()
        .filter_map(|def| match def {
            Definition::Exception(ExceptionDef { name, .. }) => Some(name.to_string()),
            Definition::ExceptionBase(ExceptionBaseDef { name, .. }) => Some(name.to_string()),
            _ => None,
        })
        .collect();

    renderer
        .tera_mut()
        .register_filter("to_class_name", move |value: &Value, _: &'_ TeraArgs| {
            let mut name = try_get_value!("to_exception_name", "value", String, value);
            name = name.to_upper_camel_case();
            if exception_names.contains(&name) {
                // Replace "Error" at the end of the name with "Exception".  Rust code typically uses
                // "Error" for any type of error but in the Java world, "Error" means a non-recoverable error
                // and is distinguished from an "Exception".
                name = match name.strip_suffix("Error") {
                    None => name,
                    Some(stripped) => format!("{}Exception", stripped),
                };
            };
            Ok(to_value(&name).unwrap())
        });

    buffer::setup_renderer(&mut renderer)?;
    class::setup_renderer(&mut renderer)?;
    collections::setup_renderer(&mut renderer)?;
    ffi::setup_renderer(&mut renderer)?;
    types::setup_renderer(&mut renderer)?;

    renderer.render_module(module)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RenderMode {
    Normal,
    TestScript,
}
