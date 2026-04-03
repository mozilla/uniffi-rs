/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::*;

pub fn map_callable(input: general::Callable, context: &Context) -> Result<Callable> {
    let fully_qualified_name_rs = fully_qualified_name_rs(&input, context)?;
    Ok(Callable {
        kind: input.kind.map_node(context)?,
        name: input.name,
        arguments: input.arguments.map_node(context)?,
        return_type: input.return_type.ty.map_node(context)?,
        fully_qualified_name_rs,
    })
}

fn fully_qualified_name_rs(callable: &general::Callable, context: &Context) -> Result<String> {
    match &callable.kind {
        general::CallableKind::Function => {
            let module_path =
                context.module_path_for_func(context.namespace_name()?, &callable.orig_name)?;
            return Ok(format!(
                "{module_path}::{}",
                names::escape_rust(&callable.name)
            ));
        }
        general::CallableKind::Method { self_type }
        | general::CallableKind::Constructor { self_type, .. }
        | general::CallableKind::VTableMethod { self_type, .. } => {
            fully_qualified_method_name_rs(&self_type.ty, callable, context)
        }
    }
}

fn fully_qualified_method_name_rs(
    self_ty: &Type,
    callable: &general::Callable,
    context: &Context,
) -> Result<String> {
    let Some(namespace) = self_ty.namespace() else {
        bail!("Invalid callable self type: {self_ty:?}");
    };
    let Some(name) = self_ty.name() else {
        bail!("Invalid callable self type: {self_ty:?}");
    };
    let module_path = context.module_path_for_type(namespace, name)?;
    let Some(self_name) = self_ty.name() else {
        bail!("Invalid Callable self type: {:?}", callable.kind);
    };
    Ok(format!(
        "{module_path}::{}::{}",
        names::escape_rust(self_name),
        names::escape_rust(&callable.name)
    ))
}

pub fn function_jni_method_name(func: &general::Function, context: &Context) -> Result<String> {
    Ok(format!(
        "function{}{}",
        context.current_crate_name()?.to_upper_camel_case(),
        func.callable.name.to_upper_camel_case()
    ))
}

pub fn constructor_jni_method_name(
    cons: &general::Constructor,
    context: &Context,
) -> Result<String> {
    let self_ty = match &cons.callable.kind {
        general::CallableKind::Constructor { self_type, .. } => self_type,
        _ => bail!("Invalid method callable kind: {:?}", cons.callable.kind),
    };
    let Some(self_name) = self_ty.ty.name() else {
        bail!("Invalid method callable kind: {:?}", cons.callable.kind);
    };
    Ok(format!(
        "constructor{}{}{}",
        context.current_crate_name()?.to_upper_camel_case(),
        self_name.to_upper_camel_case(),
        cons.callable.name.to_upper_camel_case()
    ))
}

pub fn method_jni_method_name(meth: &general::Method, context: &Context) -> Result<String> {
    let self_ty = match &meth.callable.kind {
        general::CallableKind::Method { self_type }
        | general::CallableKind::VTableMethod { self_type, .. } => self_type,
        _ => bail!("Invalid method callable kind: {:?}", meth.callable.kind),
    };
    let Some(self_name) = self_ty.ty.name() else {
        bail!("Invalid method callable kind: {:?}", meth.callable.kind);
    };
    Ok(format!(
        "method{}{}{}",
        context.current_crate_name()?.to_upper_camel_case(),
        self_name.to_upper_camel_case(),
        meth.callable.name.to_upper_camel_case()
    ))
}
