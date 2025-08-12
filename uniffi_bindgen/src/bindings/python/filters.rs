/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

/// Template filters for Askama
///
/// In general, prefer adding fields using a pipeline pass to writing filters.
/// That's allows devs to use the `pipeline` command to follow what's going on.
///
/// We currently only use filter functions when we want to implement somewhat complex display
/// logic, like in the `docstring` filter. Implementing this as a pipeline pass means the pass
/// would need to know how much each docstring gets indented, which doesn't seem right.
use askama::Result;

/// Get the idiomatic Python rendering of docstring
///
/// If the docstring is set, this returns an indented Python docstring with
/// a trailing newline. If not, it returns the empty string.
///
/// This makes it so the template code can use something like
/// `{{ item.docstring|docstring(4) -}}` to render the correct docstring in both cases.
pub fn docstring(
    docstring: &Option<String>,
    _: &dyn askama::Values,
    indent: usize,
) -> Result<String> {
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

/// Get the idiomatic Python import statement for a module
pub fn import_statement(module: &str, _: &dyn askama::Values) -> Result<String> {
    Ok(if module.starts_with('.') {
        let Some((from, name)) = module.rsplit_once('.') else {
            unreachable!()
        };
        let from = if from.is_empty() { "." } else { from };
        format!("from {from} import {name}")
    } else {
        format!("import {module}")
    })
}
