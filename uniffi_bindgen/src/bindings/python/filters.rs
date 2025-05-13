/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

/// Template filters for Askama
///
/// In general, prefer adding fields using a pipeline pass to writing filters.
/// That's allows devs to use the `pipeline` command to follow what's going on.
///
/// We currently only use filter functions for a couple reasons:
///
/// * When we want to leverage `AsRef` to implement something for any type that has an `AsRef`
///   relationship.  This could be done using a pipeline pass, but it could be annoying/distracting
///   to add fields like `type_name,` `lower_fn`, etc. to so many different Node structs.
/// * When we want to implement somewhat complex display logic, like in the `docstring` filter.
///   Implementing this as a pipeline pass means the pass would need to know how much each
///   docstring gets indented, which doesn't seem right.
use super::pipeline::nodes::*;
use askama::Result;

pub fn type_name(ty: impl AsRef<TypeNode>) -> Result<String> {
    Ok(ty.as_ref().type_name.clone())
}

pub fn ffi_converter_name(ty: impl AsRef<TypeNode>) -> Result<String> {
    Ok(ty.as_ref().ffi_converter_name.clone())
}

pub fn lower_fn(ty: impl AsRef<TypeNode>) -> Result<String> {
    Ok(format!("{}.lower", ty.as_ref().ffi_converter_name))
}

pub fn check_lower_fn(ty: impl AsRef<TypeNode>) -> Result<String> {
    Ok(format!("{}.check_lower", ty.as_ref().ffi_converter_name))
}

pub fn lift_fn(ty: impl AsRef<TypeNode>) -> Result<String> {
    Ok(format!("{}.lift", ty.as_ref().ffi_converter_name))
}

pub fn write_fn(ty: impl AsRef<TypeNode>) -> Result<String> {
    Ok(format!("{}.write", ty.as_ref().ffi_converter_name))
}

pub fn read_fn(ty: impl AsRef<TypeNode>) -> Result<String> {
    Ok(format!("{}.read", ty.as_ref().ffi_converter_name))
}

/// Get the idiomatic Python rendering of docstring
///
/// If the docstring is set, this returns an indented Python docstring with
/// a trailing newline. If not, it returns the empty string.
///
/// This makes it so the template code can use something like
/// `{{ item.docstring|docstring(4) -}}` to render the correct docstring in both cases.
pub fn docstring(docstring: &Option<String>, indent: usize) -> Result<String> {
    let Some(docstring) = docstring.as_deref() else {
        return Ok("".to_string());
    };
    let docstring = textwrap::dedent(docstring);
    let indent = " ".repeat(indent);
    // Escape triple quotes to avoid syntax error
    let escaped = docstring.replace(r#"""""#, r#"\"\"\""#);
    let indented = textwrap::indent(&escaped, &indent);
    Ok(format!("\"\"\"\n{indented}\n\"\"\"\n{indent}"))
}
