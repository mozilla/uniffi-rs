/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Add FFI scaffolding function info

use super::*;

pub fn ffi_definitions(
    namespace: &initial::Namespace,
    context: &Context,
) -> Result<Vec<FfiDefinition>> {
    let crate_name = context.crate_name()?;
    let namespace_name = &namespace.name;
    let mut ffi_definitions = vec![];

    namespace.try_visit(|func: &initial::Function| {
        let name = uniffi_meta::fn_symbol_name(&crate_name, &func.name);
        let async_data = ffi_async_data::function_async_data(func, context)?;
        let ffi_def = ffi_def(
            name,
            &func.inputs,
            func.return_type.as_ref(),
            async_data,
            context,
        )?;
        ffi_definitions.push(ffi_def);
        Ok(())
    })?;

    namespace.try_visit(|int: &initial::Interface| {
        let interface_name = int.name.clone();
        let imp = int.imp;
        let self_type = Type::Interface {
            namespace: namespace_name.to_string(),
            name: int.name.clone(),
            imp: int.imp,
        };
        int.try_visit(|meth: &initial::Method| {
            ffi_definitions.push(method_ffi_def(meth, &crate_name, &self_type, context)?);
            Ok(())
        })?;
        int.try_visit(|cons: &initial::Constructor| {
            let name =
                uniffi_meta::constructor_symbol_name(&crate_name, &interface_name, &cons.name);
            let async_data =
                ffi_async_data::constructor_async_data(cons, &interface_name, &imp, context)?;
            let ffi_def = ffi_def(name, &cons.inputs, Some(&self_type), async_data, context)?;
            ffi_definitions.push(ffi_def);
            Ok(())
        })?;
        Ok(())
    })?;
    namespace.try_visit(|record: &initial::Record| {
        let self_type = Type::Record {
            namespace: namespace_name.to_string(),
            name: record.name.clone(),
        };
        record.try_visit(|meth: &initial::Method| {
            ffi_definitions.push(method_ffi_def(meth, &crate_name, &self_type, context)?);
            Ok(())
        })?;
        Ok(())
    })?;
    namespace.try_visit(|record: &initial::Enum| {
        let self_type = Type::Enum {
            namespace: namespace_name.to_string(),
            name: record.name.clone(),
        };
        record.try_visit(|meth: &initial::Method| {
            ffi_definitions.push(method_ffi_def(meth, &crate_name, &self_type, context)?);
            Ok(())
        })?;
        Ok(())
    })?;
    Ok(ffi_definitions)
}

fn method_ffi_def(
    meth: &initial::Method,
    crate_name: &str,
    receiver_ty: &Type,
    context: &Context,
) -> Result<FfiDefinition> {
    let type_name = match receiver_ty {
        Type::CallbackInterface { name, .. }
        | Type::Interface { name, .. }
        | Type::Record { name, .. }
        | Type::Enum { name, .. }
        | Type::Custom { name, .. } => name,
        _ => bail!("invalid type"),
    };
    let name = uniffi_meta::method_symbol_name(crate_name, type_name, &meth.name);
    let async_data = ffi_async_data::method_async_data(meth, context)?;
    let mut all_args = vec![initial::Argument {
        name: "uniffi_self".to_string(),
        ty: receiver_ty.clone(),
        optional: false,
        default: None,
    }];
    all_args.extend(meth.inputs.clone());
    ffi_def(
        name,
        &all_args,
        meth.return_type.as_ref(),
        async_data,
        context,
    )
}

fn ffi_def(
    name: String,
    arguments: &[initial::Argument],
    return_type: Option<&Type>,
    async_data: Option<AsyncData>,
    context: &Context,
) -> Result<FfiDefinition> {
    let arguments: Vec<FfiArgument> = arguments
        .iter()
        .map(|a| {
            Ok(FfiArgument {
                name: a.name.clone(),
                ty: ffi_types::ffi_type(&a.ty, context)?,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(FfiDefinition::RustFunction(if async_data.is_none() {
        FfiFunction {
            name: RustFfiFunctionName(name),
            async_data: None,
            arguments,
            return_type: FfiReturnType {
                ty: return_type
                    .map(|ty| ffi_types::ffi_type(ty, context))
                    .transpose()?,
            },
            has_rust_call_status_arg: true,
            kind: FfiFunctionKind::Scaffolding,
        }
    } else {
        FfiFunction {
            name: RustFfiFunctionName(name),
            async_data,
            arguments,
            return_type: FfiReturnType {
                ty: Some(FfiType::Handle(HandleKind::RustFuture)),
            },
            has_rust_call_status_arg: false,
            kind: FfiFunctionKind::Scaffolding,
        }
    }))
}
