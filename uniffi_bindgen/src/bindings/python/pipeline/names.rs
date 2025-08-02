/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use heck::{ToShoutySnakeCase, ToSnakeCase, ToUpperCamelCase};
use once_cell::sync::Lazy;

use std::collections::HashSet;

use super::*;

// Taken from Python's `keyword.py` module.
static KEYWORDS: Lazy<HashSet<String>> = Lazy::new(|| {
    let kwlist = vec![
        "False",
        "None",
        "True",
        "__peg_parser__",
        "and",
        "as",
        "assert",
        "async",
        "await",
        "break",
        "class",
        "continue",
        "def",
        "del",
        "elif",
        "else",
        "except",
        "finally",
        "for",
        "from",
        "global",
        "if",
        "import",
        "in",
        "is",
        "lambda",
        "nonlocal",
        "not",
        "or",
        "pass",
        "raise",
        "return",
        "try",
        "while",
        "with",
        "yield",
    ];
    HashSet::from_iter(kwlist.into_iter().map(|s| s.to_string()))
});

/// Convert strings to follow Python naming guidelines
pub fn pass(module: &mut Module) -> Result<()> {
    module.visit_mut(|struct_name: &mut FfiStructName| {
        struct_name.0 = format!("_Uniffi{}", struct_name.0.to_upper_camel_case())
    });
    module.visit_mut(|function_type: &mut FfiFunctionTypeName| {
        function_type.0 = format!("_UNIFFI_{}", function_type.0.to_shouty_snake_case())
    });
    module.visit_mut(|rec: &mut Record| {
        rec.name = fixup_keyword(rec.name.to_upper_camel_case());
    });
    module.visit_mut(|e: &mut Enum| {
        e.name = fixup_keyword(e.name.to_upper_camel_case());
        match &e.shape {
            EnumShape::Error { .. } => {
                e.visit_mut(|v: &mut Variant| {
                    v.name = fixup_keyword(v.name.to_upper_camel_case());
                });
            }
            _ => {
                e.visit_mut(|v: &mut Variant| {
                    v.name = fixup_keyword(v.name.to_shouty_snake_case());
                });
            }
        }
    });
    module.visit_mut(|int: &mut Interface| {
        int.name = fixup_keyword(int.name.to_upper_camel_case());
    });
    module.visit_mut(|cbi: &mut CallbackInterface| {
        cbi.name = fixup_keyword(cbi.name.to_upper_camel_case());
    });
    module.visit_mut(|custom: &mut CustomType| {
        custom.name = fixup_keyword(custom.name.to_upper_camel_case());
    });
    module.visit_mut(|arg: &mut Argument| {
        arg.name = fixup_keyword(arg.name.to_snake_case());
    });
    module.visit_mut(|field: &mut Field| {
        field.name = fixup_keyword(field.name.to_snake_case());
    });
    module.visit_mut(|callable: &mut Callable| {
        if callable.is_primary_constructor() {
            callable.name = "__init__".to_string()
        } else {
            callable.name = fixup_keyword(callable.name.to_snake_case());
        }
    });
    module.visit_mut(|ty: &mut Type| {
        match ty {
            Type::Enum { name, .. }
            | Type::Record { name, .. }
            | Type::Interface { name, .. }
            | Type::CallbackInterface { name, .. }
            | Type::Custom { name, .. } => {
                *name = fixup_keyword(name.to_upper_camel_case());
            }
            _ => (),
        };
    });
    module.try_visit_mut(|lit: &mut Literal| {
        if let Literal::Enum(variant, ty) = lit {
            match &mut ty.ty {
                Type::Enum { name, .. } => {
                    *name = fixup_keyword(name.to_upper_camel_case());
                    // Assume enum literals are not error types and use `to_shouty_snake_case()`
                    // rather than `to_upper_camel_case()`
                    *variant = fixup_keyword(variant.to_shouty_snake_case());
                }
                type_kind => {
                    bail!("Invalid type for enum literal: {type_kind:?}")
                }
            }
        }
        Ok(())
    })?;
    Ok(())
}

/// Fixup a name by ensuring it's not a keyword
fn fixup_keyword(name: String) -> String {
    if KEYWORDS.contains(&name) {
        format!("_{name}")
    } else {
        name
    }
}
