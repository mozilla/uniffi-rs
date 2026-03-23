/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::*;

pub fn function_jni_method_name(func: &general::Function, context: &Context) -> Result<String> {
    Ok(format!(
        "function{}{}",
        context.crate_name()?.to_upper_camel_case(),
        func.callable.name.to_upper_camel_case()
    ))
}

pub fn map_kind(callable: &general::Callable, _context: &Context) -> Result<CallableKind> {
    Ok(match &callable.kind {
        general::CallableKind::Function => CallableKind::Function,
        _ => todo!(),
    })
}
