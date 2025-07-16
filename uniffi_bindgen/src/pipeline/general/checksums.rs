/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::*;
use anyhow::bail;

pub fn pass(module: &mut Module) -> Result<()> {
    // UniFFI contract version function.  This is used to check that there wasn't a breaking
    // change to `uniffi-bindgen`.
    module.ffi_uniffi_contract_version = RustFfiFunctionName(format!(
        "ffi_{}_uniffi_contract_version",
        &module.crate_name
    ));
    module.ffi_definitions.insert(
        FfiFunction {
            name: RustFfiFunctionName(format!(
                "ffi_{}_uniffi_contract_version",
                &module.crate_name
            )),
            async_data: None,
            arguments: vec![],
            return_type: FfiReturnType {
                ty: Some(FfiType::UInt32.into()),
            },
            has_rust_call_status_arg: false,
            kind: FfiFunctionKind::UniffiContractVersion,
            ..FfiFunction::default()
        }
        .into(),
    );

    // Checksums, these are used to check that the bindings were built against the same
    // exported interface as the loaded library.
    let mut checksums = vec![];
    // Closure to visit a callable
    let mut visit_callable = |callable: &Callable| {
        let Some(checksum) = callable.checksum else {
            bail!("Checksum not set for {:#?}", callable);
        };
        let fn_name = match &callable.kind {
            CallableKind::Function => {
                uniffi_meta::fn_checksum_symbol_name(&module.crate_name, &callable.name)
            }
            CallableKind::Method { self_type } => {
                let name = self_type.ty.name().unwrap();
                uniffi_meta::method_checksum_symbol_name(&module.crate_name, name, &callable.name)
            }
            CallableKind::VTableMethod { trait_name: name } => {
                uniffi_meta::method_checksum_symbol_name(&module.crate_name, name, &callable.name)
            }
            CallableKind::Constructor { interface_name, .. } => {
                uniffi_meta::constructor_checksum_symbol_name(
                    &module.crate_name,
                    interface_name,
                    &callable.name,
                )
            }
        };
        checksums.push((
            FfiFunction {
                name: RustFfiFunctionName(fn_name.clone()),
                async_data: None,
                arguments: vec![],
                return_type: FfiReturnType {
                    ty: Some(FfiType::UInt16.into()),
                },
                has_rust_call_status_arg: false,
                kind: FfiFunctionKind::Checksum,
                ..FfiFunction::default()
            },
            checksum,
        ));
        Ok(())
    };
    // Call `visit_callable` for callables that we have checksums for
    // (functions/constructors/methods), but not ones where we don't (VTable methods and UniFFI
    // traits).
    module.try_visit(|function: &Function| function.try_visit(&mut visit_callable))?;
    module.try_visit(|int: &Interface| {
        for cons in int.constructors.iter() {
            visit_callable(&cons.callable)?;
        }
        for meth in int.methods.iter() {
            visit_callable(&meth.callable)?;
        }
        Ok(())
    })?;

    for (ffi_func, checksum) in checksums {
        module.checksums.push(Checksum {
            fn_name: ffi_func.name.clone(),
            checksum,
        });
        module
            .ffi_definitions
            .insert(FfiDefinition::RustFunction(ffi_func));
    }
    module.correct_contract_version = uniffi_meta::UNIFFI_CONTRACT_VERSION.to_string();
    Ok(())
}
