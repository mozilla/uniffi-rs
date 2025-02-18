/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Rename items to match Python naming conventions

use heck::{ToShoutySnakeCase, ToSnakeCase, ToUpperCamelCase};
use once_cell::sync::Lazy;
use std::collections::HashSet;

use super::python_module_paths as prev;
use crate::nanopass::ir;

ir! {
    extend prev;

    fn mutate_type(ty: &mut Type) {
        match ty {
            Type::Enum { name, .. }
            | Type::Record { name, .. }
            | Type::Interface { name, .. }
            | Type::CallbackInterface { name, .. }
            | Type::Custom { name, .. } => {
                *name = class_name(name);
            }
            _ => ()
        }
    }

    fn mutate_record(rec: &mut Record) {
        rec.name = class_name(&rec.name);
    }

    fn mutate_interface(int: &mut Interface) {
        int.name = class_name(&int.name);
        for trait_impl in int.trait_impls.iter_mut() {
            trait_impl.trait_name = class_name(&trait_impl.trait_name);
        }
    }

    fn mutate_callbackinterface(cbi: &mut CallbackInterface) {
        cbi.name = class_name(&cbi.name);
    }

    fn mutate_enum(en: &mut Enum) {
        en.name = class_name(&en.name);
        let enum_is_error = matches!(en.shape, EnumShape::Error { .. });
        for variant in en.variants.iter_mut() {
            variant.name = if enum_is_error {
                class_name(&variant.name)
            } else {
                enum_variant_name(&variant.name)
            }
        }
    }

    fn mutate_custom_type(custom: &mut CustomType) {
        custom.name = class_name(&custom.name);
    }

    fn mutate_callable(callable: &mut Callable) {
        callable.name = fn_name(&callable.name);
    }

    fn mutate_argument(arg: &mut Argument) {
        arg.name = var_name(&arg.name);
    }

    fn mutate_field(field: &mut Field) {
        field.name = var_name(&field.name);
    }

    fn mutate_literal(lit: &mut Literal) -> Result<()> {
        match lit {
            Literal::Enum(variant, ty) => match &mut ty.ty {
                Type::Enum { name, .. } => {
                    *name = class_name(name);
                    *variant = enum_variant_name(variant);
                }
                type_kind => {
                    bail!("Invalid type for enum literal: {type_kind:?}")
                }
            },
            _ => ()
        }
        Ok(())
    }

    fn mutate_ffi_type(ffi_type: &mut FfiType) {
        match ffi_type {
            FfiType::Function(name) => {
                *name = ffi_function_name(&name);
            }
            FfiType::Struct(name) => {
                *name = ffi_struct_name(&name);
            }
            _ => ()
        }
    }

    fn mutate_ffi_definition(ffi_def: &mut FfiDefinition) {
        match ffi_def {
            FfiDefinition::FunctionType(func) => {
                func.name = ffi_function_name(&func.name);
            }
            FfiDefinition::Struct(st) => {
                st.name = ffi_struct_name(&st.name);
            }
            // Don't change these names.  We need to use the exact symbol name from the dylib.
            FfiDefinition::RustFunction(_) => (),
        }
    }
}

/// Idiomatic Python class name (for enums, records, errors, etc).
fn class_name(nm: &str) -> String {
    fixup_keyword(nm.to_string().to_upper_camel_case())
}

/// Idiomatic Python function name.
fn fn_name(nm: &str) -> String {
    fixup_keyword(nm.to_string().to_snake_case())
}

/// Idiomatic Python a variable name.
fn var_name(nm: &str) -> String {
    fixup_keyword(nm.to_string().to_snake_case())
}

/// Idiomatic Python enum variant name.
///
/// These use SHOUTY_SNAKE_CASE style.  User's access them through `EnumName.VARIANT_NAME`.
/// This naming style is used for both flat enums and fielded ones.
///
/// The exception is error enums.  In that case, there's no enum in the generated python.
/// Instead there's a base class and subclasses -- all with UpperCamelCase names.
/// That case is not handled by this method.
fn enum_variant_name(nm: &str) -> String {
    fixup_keyword(nm.to_string().to_shouty_snake_case())
}

/// Idiomatic name for an FFI function type.
///
/// These follow the ctypes convention of SHOUTY_SNAKE_CASE. Prefix with `_UNIFFI` so that
/// it can't conflict with user-defined items.
fn ffi_function_name(name: &str) -> String {
    format!("_UNIFFI_FN_{}", name.to_shouty_snake_case())
}

/// Idiomatic name for an FFI struct.
///
/// These follow the ctypes convention of UpperCamelCase (although the ctypes docs also uses
/// SHOUTY_SNAKE_CASE in some places).  Prefix with `_Uniffi` so that it can't conflict with
/// user-defined items.
fn ffi_struct_name(name: &str) -> String {
    format!("_UniffiStruct{}", name.to_upper_camel_case())
}

/// Fixup a name by ensuring it's not a keyword
fn fixup_keyword(name: String) -> String {
    if KEYWORDS.contains(&name) {
        format!("_{name}")
    } else {
        name
    }
}

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
