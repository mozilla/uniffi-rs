/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::ir::{ClassName, EnumDef, Module, VarName};
use heck::{
    ToKebabCase, ToLowerCamelCase, ToShoutyKebabCase, ToShoutySnakeCase, ToSnakeCase,
    ToUpperCamelCase,
};
use regex::Captures;
use serde::Serialize;
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Weak};
use tera::{to_value, try_get_value, Context, Error, Map, Result, Tera, Value};

pub type TeraArgs = HashMap<std::string::String, Value>;

// Copied from the once-cell docs
macro_rules! regex {
    ($re:literal $(,)?) => {{
        static RE: once_cell::sync::OnceCell<regex::Regex> = once_cell::sync::OnceCell::new();
        RE.get_or_init(|| regex::Regex::new($re).unwrap())
    }};
}

/// Renders AST items
///
/// This struct wraps Tera and adds some extra features:
///   - Registers some useful functions and filters (see [setup_tera])
///   - Supports "AST templates", which are tera templates that can recursively render AST items
///     (see [Self::add_ast_templates])
///   - Adds tera functions to lookup definitions from a [Module] (see [setup_definition_getters])
pub struct Renderer {
    tera: Tera,
}

impl Renderer {
    pub fn new() -> Self {
        let mut tera = Tera::default();
        setup_tera(&mut tera);
        Self { tera }
    }

    pub fn tera(&mut self) -> &Tera {
        &self.tera
    }

    pub fn tera_mut(&mut self) -> &mut Tera {
        &mut self.tera
    }

    /// Add AST templates
    ///
    /// AST templates are Tera templates that get pre-processing make recursive rendering simpler.
    /// Templates are passed in as (`ir_type_name`, `template_source`) tuples.  `ir_type_name` is
    /// the `ast_type` field set by our `serde::Serialize` implementations.  `template_source` is
    /// usually either a very short string or comes from an `include_str!` call.
    ///
    /// In the template source other AST items can be recursively rendered by putting them in a
    /// Tera expression.  For example the Add expression can be defined using
    /// `"{{ first }} + {{ second }}"`.
    ///
    /// If an expression appears by itself on a line and is rendered as multiple lines, each
    /// non-empty line after the first will get an extra indent based on the position of the first
    /// line.  This allows blocks to be rendered correctly in lanugages like Python where the
    /// indentation is part of the syntax.  For example:
    ///
    /// ```jinja
    /// for {{ var }} in {{ expression }}:
    ///     {{ block }}
    /// ```
    ///
    /// Finally, if an recursively rendered expression ends in a newline, it will be trimmed.  This
    /// is almost always what you want.  In the `Add` example you don't want a newline after the
    /// sub-expressions and in the `For` example you dont want an 2 newlines after the block.  Only
    /// 1 newline will be trimmed, which allows for templates to end in a newline if they really
    /// want to.
    pub fn add_ast_templates(
        &mut self,
        templates: impl IntoIterator<Item = (&'static str, &'static str)>,
    ) -> Result<()> {
        self.tera.add_raw_templates(
            templates
                .into_iter()
                .map(|(name, source)| (name, add_implicit_render_filters(source))),
        )
    }

    /// Render an AST module
    pub fn render_module(&mut self, module: Module) -> Result<String> {
        let serialized_module = to_value(&module)?;
        self.render_value(module, &serialized_module)
    }

    // Does the work for render_module, but is split out for easier testing
    fn render_value(&mut self, module: Module, value: &Value) -> Result<String> {
        setup_definition_getters(&mut self.tera, module);

        // There's a very annoying circular reference between Tera and the render filter.  To work
        // around that we:
        //   - Put our Tera into an `Arc<>`
        //   - Give the render filter a weakref to Tera.  This creates the circular reference, so
        //     we need to use Arc::new_cyclic to make this work.
        //   - We still need a `Tera` value inside the `Renderer` struct.  Put a dummy value
        //     there while the rendering is happened, then unwrap the Arc at the end of everything
        //     to restore it.
        let arc_tera = Arc::new_cyclic(|weakref| {
            setup_render_filters(&mut self.tera, weakref);
            std::mem::take(&mut self.tera)
        });
        let result = render_value(&arc_tera, value);
        self.tera = Arc::try_unwrap(arc_tera).unwrap();
        result
    }
}

fn add_implicit_render_filters(template_source: &str) -> String {
    let expression_re = regex!(r"(\{\{\s*)(.*?)(\s*}})");
    // Matches a line with just one expression
    let single_expression_re = regex!(r"^(\s*)(\{\{\s*)([^}]*?)(\s*}})\s*$");

    template_source
        .split('\n')
        .map(|line| {
            if let Some(caps) = single_expression_re.captures(line) {
                // If a line consists of a single expression only, pass an indent to render so that
                // subsequent lines are indented
                Cow::from(format!(
                    "{}{}{} | render(indent={}){}",
                    &caps[1],
                    &caps[2],
                    &caps[3],
                    &caps[1].chars().count(),
                    &caps[4]
                ))
            } else {
                expression_re.replace_all(line, |caps: &Captures| {
                    format!("{}{} | render{}", &caps[1], &caps[2], &caps[3])
                })
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Add filters and functions that are generally useful for AST rendering
///
/// This includes:
///   - Heck-based case functions (`to_lower_camel_case`, `to_snake_case`, etc).
///   - The `has_fields()` filter, which forwards to `EnumDef::has_fields`
///   - The zip function which zips items from multiple vecs together.  Each argument should be a
///     Value::Array.  A new array will be created with items from each argument, where each item
///     is an `Value::Object` with the items from each array.  The field names will match the
///     argument names passed in to `zip()`.
///   - The `temp_var_name` function which outputs a VarName that should not collide with other
///     names.
pub fn setup_tera(tera: &mut Tera) {
    tera.register_filter("to_kebab_case", |value: &Value, _: &'_ TeraArgs| {
        let s = try_get_value!("to_kebab_case", "value", String, value);
        Ok(to_value(&s.to_kebab_case()).unwrap())
    });
    tera.register_filter("to_lower_camel_case", |value: &Value, _: &'_ TeraArgs| {
        let s = try_get_value!("to_lower_camel_case", "value", String, value);
        Ok(to_value(&s.to_lower_camel_case()).unwrap())
    });
    tera.register_filter("to_shouty_kebab_case", |value: &Value, _: &'_ TeraArgs| {
        let s = try_get_value!("to_shouty_kebab_case", "value", String, value);
        Ok(to_value(&s.to_shouty_kebab_case()).unwrap())
    });
    tera.register_filter("to_shouty_snake_case", |value: &Value, _: &'_ TeraArgs| {
        let s = try_get_value!("to_shouty_snake_case", "value", String, value);
        Ok(to_value(&s.to_shouty_snake_case()).unwrap())
    });
    tera.register_filter("to_snake_case", |value: &Value, _: &'_ TeraArgs| {
        let s = try_get_value!("to_snake_case", "value", String, value);
        Ok(to_value(&s.to_snake_case()).unwrap())
    });
    tera.register_filter("to_upper_camel_case", |value: &Value, _: &'_ TeraArgs| {
        let s = try_get_value!("to_upper_camel_case", "value", String, value);
        Ok(to_value(&s.to_upper_camel_case()).unwrap())
    });

    tera.register_filter("has_fields", |value: &'_ Value, _: &_| {
        let enum_ = try_get_value!("has_fields", "value", EnumDef, value);
        Ok(to_value(enum_.has_fields())?)
    });

    tera.register_function("zip", |args: &'_ TeraArgs| {
        let mut iters = vec![];
        for (name, value) in args {
            let v = try_get_value!("zip", name, Vec<Value>, value);
            iters.push((name, v.into_iter()));
        }
        let mut ziped_values = vec![];
        'outer: loop {
            let mut item = Map::new();
            for (name, ref mut iter) in &mut iters {
                match iter.next() {
                    Some(v) => item.insert(name.to_string(), v),
                    None => break 'outer,
                };
            }
            ziped_values.push(Value::Object(item));
        }

        Ok(Value::Array(ziped_values))
    });

    tera.register_function("temp_var_name", |_: &_| {
        static COUNTER: AtomicU32 = AtomicU32::new(0);
        let val = COUNTER.fetch_add(1, Ordering::Relaxed);
        Ok(to_value(VarName::new(format!("bindingsIrTmp{val}")))?)
    });
}

/// Setup definition getter functions.
///
/// This registers functions that fetch definition items from the Module, for example `get_class`,
/// `get_function`, etc.
///
/// Also the `variant` filter will be added, which can be used to lookup a variant from an
/// `EnumDef`.  The name of the variant should be passed in as the `name` argument.
///
/// Each call to [Renderer.render_module()] will call this, replaces any previous registered
/// definition getters.
pub fn setup_definition_getters(tera: &mut Tera, module: Module) {
    // Create an Arc to the module for each DefinitionGetter to share
    let module = Arc::new(module);
    setup_definition_getter(tera, &module, "get_cstruct", |module, name| {
        module.get_cstruct(name)
    });
    setup_definition_getter(tera, &module, "get_function", |module, name| {
        module.get_function(name)
    });
    setup_definition_getter(tera, &module, "get_class", |module, name| {
        module.get_class(name)
    });
    setup_definition_getter(tera, &module, "get_enum", |module, name| {
        module.get_enum(name)
    });
    setup_definition_getter(tera, &module, "get_cstruct", |module, name| {
        module.get_cstruct(name)
    });
    setup_definition_getter(tera, &module, "get_exception_base", |module, name| {
        module.get_exception_base(name)
    });
    setup_definition_getter(tera, &module, "get_exception", |module, name| {
        module.get_exception(name)
    });
    setup_definition_getter(tera, &module, "get_def", |module, name| {
        module.get_def(name)
    });
    setup_definition_getter(tera, &module, "get_ffi_function", |module, name| {
        module
            .native_library
            .as_ref()
            .map(|lib| lib.functions.get(name))
            .flatten()
    });
    tera.register_filter("variant", move |value: &Value, args: &'_ TeraArgs| {
        let enum_ = try_get_value!("variant", "value", EnumDef, value);
        let variant_name = try_get_value!(
            "variant",
            "name",
            ClassName,
            args.get("name").unwrap_or(&Value::Null)
        );
        let variant = enum_.get_variant(&variant_name.name);
        match variant {
            Some(v) => Ok(to_value(v)?),
            None => Err(Error::msg(format!("Variant not found: {variant_name}"))),
        }
    });
}

fn setup_definition_getter<F, T>(
    tera: &mut Tera,
    module: &Arc<Module>,
    func_name: &'static str,
    getter: F,
) where
    F: for<'a, 'b> Fn(&'a Module, &'b str) -> Option<&'a T> + Send + Sync + 'static,
    T: Serialize + 'static,
{
    let module = module.clone();
    tera.register_function(func_name, move |args: &TeraArgs| {
        let name = match args.get("name") {
            Some(Value::String(s)) => s,
            Some(Value::Object(map)) => match map.get("name") {
                Some(Value::String(s)) => s,
                Some(_) => {
                    return Err(Error::msg(format!(
                    "Function {func_name} was called with an object with a non-string `name` field",
                )))
                }
                None => {
                    return Err(Error::msg(format!(
                        "Function {func_name} was called with an object without a `name` field",
                    )))
                }
            },
            Some(val) => {
                return Err(Error::msg(format!(
                "Function {func_name} was called with an invalid type for arg name (got `{val}`)",
            )))
            }
            None => {
                return Err(Error::msg(format!(
                    "Function {func_name} was called without a name arg",
                )))
            }
        };
        match getter(module.as_ref(), name) {
            Some(v) => Ok(to_value(v)?),
            None => Err(Error::msg(format!("{name} not defined"))),
        }
    });
}

fn setup_render_filters(tera: &mut Tera, weakref: &Weak<Tera>) {
    let render_weakref = Weak::clone(weakref);
    tera.register_filter("render", move |value: &Value, args: &'_ TeraArgs| {
        let tera = render_weakref
            .upgrade()
            .ok_or_else(|| Error::msg("Weakref no longer valid"))?;
        let indent_str = args
            .get("indent")
            .map(|v| " ".repeat(v.as_u64().unwrap_or(0) as usize));
        render_value(&tera, value).map(|s| match indent_str {
            Some(indent_str) => {
                let mut s = s
                    .split('\n')
                    .enumerate()
                    .map(|(i, line)| {
                        if i == 0 || line.is_empty() {
                            Cow::from(line)
                        } else {
                            Cow::from(format!("{}{}", indent_str, line))
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                if let Some('\n') = s.chars().last() {
                    s.pop();
                }
                Value::String(s)
            }
            None => Value::String(s),
        })
    });
    // Also register comma_join, which depends on render
    let comma_join_weakref = weakref.clone();
    tera.register_filter("comma_join", move |value: &Value, _: &'_ TeraArgs| {
        let tera = comma_join_weakref
            .upgrade()
            .ok_or_else(|| Error::msg("Weakref no longer valid"))?;
        let values = try_get_value!("comma_join", "value", Vec<Value>, value);
        Ok(Value::String(
            values
                .into_iter()
                .map(|v| {
                    render_value(&tera, &v).map(|mut s| {
                        if let Some('\n') = s.chars().last() {
                            s.pop();
                        }
                        s
                    })
                })
                .collect::<Result<Vec<String>>>()?
                .join(", "),
        ))
    });
}

fn render_value(tera: &Tera, value: &Value) -> Result<String> {
    Ok(match value {
        Value::String(v) => v.clone(),
        Value::Number(v) => v.to_string(),
        Value::Bool(v) => v.to_string(),
        Value::Null => "null".to_string(),
        Value::Array(_) => "[array]".to_string(),
        Value::Object(o) => {
            if let Some(Value::String(ir_type)) = o.get("ir_type") {
                let mut context = Context::from_value(value.clone())?;
                context.insert("self", value);
                tera.render(ir_type, &context)?
            } else {
                "[object]".to_string()
            }
        }
    })
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::ast::helpers::*;

    // Render a single AST item instead of a whole module, which makes the tests simpler
    fn render_ast_item(renderer: &mut Renderer, item: impl Serialize) -> String {
        render_ast_item_with_module(renderer, Module::new(), item)
    }

    fn render_ast_item_with_module(
        renderer: &mut Renderer,
        module: Module,
        item: impl Serialize,
    ) -> String {
        renderer
            .render_value(module, &to_value(&item).unwrap())
            .unwrap()
    }

    #[test]
    fn test_add_implicit_render_filters() {
        let test_cases = [
            ("no_substitutions_needed", "no_substitutions_needed"),
            ("{{ foo }}", "{{ foo | render(indent=0) }}"),
            ("{{ foo.bar }}", "{{ foo.bar | render(indent=0) }}"),
            (
                "{{ foo.bar|filter }}",
                "{{ foo.bar|filter | render(indent=0) }}",
            ),
            (
                r#"{{ foo.bar | replace(from="a", "to"b") }}"#,
                r#"{{ foo.bar | replace(from="a", "to"b") | render(indent=0) }}"#,
            ),
            (
                "{{ foo }}\n{{ bar }}",
                "{{ foo | render(indent=0) }}\n{{ bar | render(indent=0) }}",
            ),
            ("    {{ foo }}", "    {{ foo | render(indent=4) }}"),
            (
                "    {{ foo }} {{ bar }}",
                "    {{ foo | render }} {{ bar | render }}",
            ),
            (
                "{{ dont_add_render \n when_split_over_lines }}",
                "{{ dont_add_render \n when_split_over_lines }}",
            ),
        ];
        for (source, rendered) in test_cases {
            assert_eq!(add_implicit_render_filters(source), rendered)
        }
    }

    #[test]
    fn test_renderer() {
        let mut renderer = Renderer::new();
        renderer
            .add_ast_templates([
                ("Add", "{{ first }} + {{ second }}"),
                ("Var", "{{ name }}"),
                ("VarName", "{{ name|to_lower_camel_case }}"),
            ])
            .unwrap();

        assert_eq!(
            render_ast_item(
                &mut renderer,
                &add(ident("number_one"), ident("number_two"))
            ),
            "numberOne + numberTwo"
        )
    }

    #[test]
    fn test_definition_getters() {
        let mut renderer = Renderer::new();
        renderer.add_ast_templates([
            (
                "ClassName",
                "{%- set cstruct_def = get_cstruct(name=name) -%}{{ cstruct_def.fields|map(attribute='name')|comma_join }}"
            ),
        ])
        .unwrap();
        let mut module = Module::new();
        module.add_cstruct(cstruct::def(
            "MyStruct",
            [
                cstruct::field("field_one", types::int32()),
                cstruct::field("field_two", types::int32()),
            ],
        ));

        assert_eq!(
            render_ast_item_with_module(&mut renderer, module, &ClassName::new("MyStruct")),
            "field_one, field_two"
        )
    }

    #[test]
    fn test_comma_join() {
        let mut renderer = Renderer::new();
        renderer
            .add_ast_templates([
                ("Var", "{{ name }}"),
                ("VarName", "{{ name|to_lower_camel_case }}"),
                ("FunctionName", "{{ name|to_lower_camel_case }}"),
                ("Call", "{{ name }}({{ values|comma_join }})"),
            ])
            .unwrap();

        assert_eq!(
            render_ast_item(
                &mut renderer,
                &call("foo", [ident("x"), ident("y"), ident("z")])
            ),
            "foo(x, y, z)"
        )
    }

    #[test]
    fn test_zip() {
        let mut renderer = Renderer::new();
        let rendered = renderer.tera_mut().render_str(
            r#"{% for item in zip(one=["a", "b", "c"], two=[1, 2, 3]) %}{{ item.one }}-{{ item.two }}{% if not loop.last %}, {% endif %}{% endfor %}"#,
            &Context::new(),
        ).unwrap();
        assert_eq!(rendered, "a-1, b-2, c-3",)
    }

    #[test]
    fn test_indent_expressions() {
        let mut renderer = Renderer::new();
        renderer
            .add_ast_templates([
                ("FunctionName", "{{ name|to_lower_camel_case }}"),
                ("VarName", "{{ name|to_lower_camel_case }}"),
                ("LiteralInt", "{{ value }}"),
                ("Init", "{{ name }} = {{ initial }}"),
                ("Var", "{{ name }}"),
                ("Return", "return {{ value }}"),
                ("FunctionDef", "def {{ name }}():\n    {{ body }}"),
                (
                    "Block",
                    "{% for stmt in statements %}{{ stmt }}\n{% endfor %}",
                ),
            ])
            .unwrap();

        assert_eq!(
            render_ast_item(
                &mut renderer,
                &func("foo").body([var("x", types::int(), lit::int(1)), return_(ident("x")),])
            ),
            vec!["def foo():", "    x = 1", "    return x",].join("\n")
        )
    }

    #[test]
    fn test_remove_trailing_newline() {
        let mut renderer = Renderer::new();
        renderer
            .add_ast_templates([
                ("Var", "{{ name }}"),
                ("VarName", "{{ name|to_lower_camel_case }}\n"),
            ])
            .unwrap();

        assert_eq!(
            render_ast_item(&mut renderer, ident("number_one")),
            "numberOne",
        )
    }
}
