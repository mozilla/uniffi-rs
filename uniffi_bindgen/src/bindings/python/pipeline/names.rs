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

/// Fixup a name by ensuring it's not a keyword
fn fixup_keyword(name: String) -> String {
    if KEYWORDS.contains(&name) {
        format!("_{name}")
    } else {
        name
    }
}

pub fn map_ffi_function_type_name(
    ffi_function_type_name: FfiFunctionTypeName,
    _: &Context,
) -> Result<FfiFunctionTypeName> {
    Ok(FfiFunctionTypeName(function_type_name(
        &ffi_function_type_name.0,
    )))
}

pub fn map_ffi_struct_name(ffi_struct_name: FfiStructName, _: &Context) -> Result<FfiStructName> {
    Ok(FfiStructName(struct_name(&ffi_struct_name.0)))
}

pub fn function_type_name(name: &str) -> String {
    format!("_UNIFFI_{}", name.to_shouty_snake_case())
}

pub fn struct_name(name: &str) -> String {
    format!("_Uniffi{}", name.to_upper_camel_case())
}

pub fn type_name(name: &str) -> String {
    fixup_keyword(name.to_upper_camel_case())
}

pub fn var_name(name: &str) -> String {
    fixup_keyword(name.to_snake_case())
}

pub fn function_name(name: &str) -> String {
    fixup_keyword(name.to_snake_case())
}

pub fn non_error_variant_name(name: &str) -> String {
    // Non error variants get SHOUTY_CASE_NAMES
    fixup_keyword(name.to_shouty_snake_case())
}
