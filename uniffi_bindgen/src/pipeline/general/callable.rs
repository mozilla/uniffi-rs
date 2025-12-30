/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Extract common data from Function/Method/Constructor into Callable

use super::ffi_async_data::{constructor_async_data, function_async_data, method_async_data};
use super::*;

pub fn function_callable(func: &initial::Function, context: &Context) -> Result<Callable> {
    let kind = CallableKind::Function;
    let arguments = map_func_args(&func.inputs, &func.name, context)?;
    let name = rename::func(func.name.clone(), context)?;

    Ok(Callable {
        name,
        async_data: function_async_data(func, context)?,
        kind,
        arguments,
        return_type: ReturnType {
            ty: func.return_type.clone().map_node(context)?,
        },
        throws_type: ThrowsType {
            ty: func.throws.clone().map_node(context)?,
        },
        checksum: func.checksum,
        ffi_func: RustFfiFunctionName(uniffi_meta::fn_symbol_name(
            &context.crate_name()?,
            &func.name,
        )),
    })
}

pub fn method_callable(meth: &initial::Method, context: &Context) -> Result<Callable> {
    let self_type = context.self_type()?;
    let ffi_func = RustFfiFunctionName(uniffi_meta::method_symbol_name(
        &context.crate_name()?,
        &context.current_type_name()?,
        &meth.name,
    ));
    let kind = CallableKind::Method { self_type };
    let arguments = map_method_args(&meth.inputs, &meth.name, context)?;
    let name = rename::method(meth.name.clone(), context)?;

    Ok(Callable {
        name,
        arguments,
        return_type: ReturnType {
            ty: meth.return_type.clone().map_node(context)?,
        },
        throws_type: ThrowsType {
            ty: meth.throws.clone().map_node(context)?,
        },
        checksum: meth.checksum,
        async_data: method_async_data(meth, context)?,
        ffi_func,
        kind,
    })
}

pub fn constructor_callable(cons: &initial::Constructor, context: &Context) -> Result<Callable> {
    let self_type = context.self_type()?;
    let ffi_func = RustFfiFunctionName(uniffi_meta::constructor_symbol_name(
        &context.crate_name()?,
        &context.current_type_name()?,
        &cons.name,
    ));
    let (interface_name, imp) = match &self_type.ty {
        Type::Interface { name, imp, .. } => (name, imp),
        _ => bail!("Invalid self type for constructor: {self_type:?}"),
    };
    let kind = CallableKind::Constructor {
        primary: cons.name == "new",
        self_type: self_type.clone(),
    };
    let arguments = map_method_args(&cons.inputs, &cons.name, context)?;
    let name = rename::method(cons.name.clone(), context)?;

    Ok(Callable {
        name,
        async_data: constructor_async_data(cons, interface_name, imp, context)?,
        arguments,
        return_type: ReturnType {
            ty: Some(self_type),
        },
        throws_type: ThrowsType {
            ty: cons.throws.clone().map_node(context)?,
        },
        checksum: cons.checksum,
        ffi_func,
        kind,
    })
}

pub fn map_func_args(
    inputs: &[initial::Argument],
    fn_name: &str,
    context: &Context,
) -> Result<Vec<Argument>> {
    inputs
        .iter()
        .cloned()
        .map(|arg| {
            let mut child_context = context.clone();
            let context = &mut child_context;

            context.update_from_arg(&arg)?;
            Ok(Argument {
                name: rename::func_arg(arg.name, fn_name, context)?,
                ty: arg.ty.map_node(context)?,
                optional: arg.optional,
                default: arg.default.map_node(context)?,
            })
        })
        .collect()
}

pub fn map_method_args(
    inputs: &[initial::Argument],
    fn_name: &str,
    context: &Context,
) -> Result<Vec<Argument>> {
    inputs
        .iter()
        .cloned()
        .map(|arg| {
            let mut child_context = context.clone();
            let context = &mut child_context;

            context.update_from_arg(&arg)?;
            Ok(Argument {
                name: rename::method_arg(arg.name, fn_name, context)?,
                ty: arg.ty.map_node(context)?,
                optional: arg.optional,
                default: arg.default.map_node(context)?,
            })
        })
        .collect()
}
