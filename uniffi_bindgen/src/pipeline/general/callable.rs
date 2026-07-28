/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Extract common data from Function/Method/Constructor into Callable

use super::ffi_async_data::{constructor_async_data, function_async_data, method_async_data};
use super::*;

/// Zero-copy `&mut [u8]` / `[ByMutRef]` is unsound across an async boundary:
/// the Rust future can resume on another thread after the foreign caller has
/// continued and freed the buffer, leaving the `&mut` borrow dangling.
///
/// The proc-macro (`uniffi_macros`) and UDL (`uniffi_udl`) frontends reject
/// this at their own layer; this guard also covers the source-parsing frontend
/// (`uniffi_parse_rs`), which has no earlier check.
fn reject_async_by_mut_ref(is_async: bool, name: &str, arguments: &[Argument]) -> Result<()> {
    if is_async && arguments.iter().any(|arg| arg.is_borrowed_bytes_mut()) {
        bail!(
            "`&mut [u8]` / `[ByMutRef]` arguments are not supported in async functions: \
             the future may resume after the caller frees the buffer (in `{name}`)",
        );
    }
    Ok(())
}

pub fn function_callable(func: &initial::Function, context: &Context) -> Result<Callable> {
    let kind = CallableKind::Function;
    let arguments = map_func_args(&func.inputs, &func.name, context)?;
    let name = rename::func(func.name.clone(), context)?;

    let callable = Callable {
        id: func.id,
        name,
        orig_name: func.orig_name.clone(),
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
    };
    reject_async_by_mut_ref(
        callable.async_data.is_some(),
        &callable.orig_name,
        &callable.arguments,
    )?;
    Ok(callable)
}

pub fn method_callable(meth: &initial::Method, context: &Context) -> Result<Callable> {
    let self_type = context.self_type()?;
    method_callable_with_kind(
        meth,
        CallableKind::Method {
            self_type,
            takes_self_by_arc: meth.takes_self_by_arc,
        },
        context,
    )
}

pub fn method_callable_with_kind(
    meth: &initial::Method,
    kind: CallableKind,
    context: &Context,
) -> Result<Callable> {
    let ffi_func = RustFfiFunctionName(uniffi_meta::method_symbol_name(
        &context.crate_name()?,
        &context.current_type_name()?,
        &meth.name,
    ));
    let arguments = map_method_args(&meth.inputs, &meth.name, context)?;
    let name = rename::method(meth.name.clone(), context)?;

    let callable = Callable {
        id: meth.id,
        name,
        orig_name: meth.orig_name.clone(),
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
    };
    reject_async_by_mut_ref(
        callable.async_data.is_some(),
        &callable.orig_name,
        &callable.arguments,
    )?;
    Ok(callable)
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

    let callable = Callable {
        id: cons.id,
        name,
        orig_name: cons.orig_name.clone(),
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
    };
    reject_async_by_mut_ref(
        callable.async_data.is_some(),
        &callable.orig_name,
        &callable.arguments,
    )?;
    Ok(callable)
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
                orig_name: arg.name.clone(),
                name: rename::func_arg(arg.name, fn_name, context)?,
                ty: arg.ty.map_node(context)?,
                by_ref: arg.by_ref,
                by_mut_ref: arg.by_mut_ref,
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
                orig_name: arg.name.clone(),
                name: rename::method_arg(arg.name, fn_name, context)?,
                ty: arg.ty.map_node(context)?,
                by_ref: arg.by_ref,
                by_mut_ref: arg.by_mut_ref,
                optional: arg.optional,
                default: arg.default.map_node(context)?,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bytes_arg(by_mut_ref: bool) -> Argument {
        Argument {
            name: "buf".to_string(),
            orig_name: "buf".to_string(),
            ty: TypeNode {
                id: 0,
                canonical_name: "Bytes".to_string(),
                is_used_as_error: false,
                has_from_unexpected_callback_error_impl: false,
                ffi_type: FfiType::RustBuffer(None),
                ty: Type::Bytes,
            },
            by_ref: true,
            by_mut_ref,
            optional: false,
            default: None,
        }
    }

    #[test]
    fn async_mut_ref_bytes_is_rejected() {
        // `&mut [u8]` in an async fn must be rejected.
        assert!(reject_async_by_mut_ref(true, "fill", &[bytes_arg(true)]).is_err());
    }

    #[test]
    fn sync_mut_ref_bytes_is_allowed() {
        // `&mut [u8]` in a sync fn is fine.
        assert!(reject_async_by_mut_ref(false, "fill", &[bytes_arg(true)]).is_ok());
    }

    #[test]
    fn async_ref_bytes_is_allowed() {
        // Read-only `&[u8]` in an async fn is fine — only `&mut` is unsound.
        assert!(reject_async_by_mut_ref(true, "sum", &[bytes_arg(false)]).is_ok());
    }
}
