/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Rename items to match Python naming conventions

use super::names as prev;
use crate::nanopass::ir;

ir! {
    extend prev;

    fn mutate_module(module: &mut Module) {
        format_docstring(&mut module.docstring);
    }

    fn mutate_record(rec: &mut Record) {
        format_docstring(&mut rec.docstring);
    }

    fn mutate_enum(en: &mut Enum) {
        format_docstring(&mut en.docstring);
    }

    fn mutate_interface(int: &mut Interface) {
        format_docstring(&mut int.docstring);
    }

    fn mutate_callback_interface(cbi: &mut CallbackInterface) {
        format_docstring(&mut cbi.docstring);
    }

    fn mutate_custom_type(custom: &mut CustomType) {
        format_docstring(&mut custom.docstring);
    }

    fn mutate_variant(var: &mut Variant) {
        format_docstring(&mut var.docstring);
    }

    fn mutate_field(field: &mut Field) {
        format_docstring(&mut field.docstring);
    }

    fn mutate_function(func: &mut Function) {
        format_docstring(&mut func.docstring);
    }

    fn mutate_method(meth: &mut Method) {
        format_docstring(&mut meth.docstring);
    }

    fn mutate_constructor(cons: &mut Constructor) {
        format_docstring(&mut cons.docstring);
    }
}

pub fn format_docstring(docstring: &mut Option<String>) {
    if let Some(docstring) = docstring {
        // Escape triple quotes to avoid syntax errors in docstrings
        *docstring = docstring.replace(r#"""""#, r#"\"\"\""#);
        // Remove indentation and surround with quotes
        *docstring = format!("\"\"\"\n{}\n\"\"\"", &textwrap::dedent(&docstring));
    }
}
