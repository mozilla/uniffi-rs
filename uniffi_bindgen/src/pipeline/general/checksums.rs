/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::*;

// Checksums, these are used to check that the bindings were built against the same
// exported interface as the loaded library.
pub fn checksums(namespace: &initial::Namespace) -> Result<Vec<Checksum>> {
    let mut checksums = vec![];

    namespace.visit(|func: &initial::Function| {
        let Some(checksum) = func.checksum else {
            return;
        };
        checksums.push(Checksum {
            checksum,
            fn_name: RustFfiFunctionName(uniffi_meta::fn_checksum_symbol_name(
                &namespace.crate_name,
                &func.name,
            )),
        });
    });

    namespace.visit(|int: &initial::Interface| {
        let interface_name = int.name.clone();
        // Note: we're specifically only visiting `int.methods` here.  This skips methods for
        // traits like `Debug` which live in the `uniffi_traits` vec and don't have checksums.
        int.methods.visit(|meth: &initial::Method| {
            let Some(checksum) = meth.checksum else {
                return;
            };
            checksums.push(Checksum {
                checksum,
                fn_name: RustFfiFunctionName(uniffi_meta::method_checksum_symbol_name(
                    &namespace.crate_name,
                    &interface_name,
                    &meth.name,
                )),
            });
        });
        int.visit(|cons: &initial::Constructor| {
            let Some(checksum) = cons.checksum else {
                return;
            };
            checksums.push(Checksum {
                checksum,
                fn_name: RustFfiFunctionName(uniffi_meta::constructor_checksum_symbol_name(
                    &namespace.crate_name,
                    &interface_name,
                    &cons.name,
                )),
            });
        });
    });

    // Skip callback interfaces, since those don't get their checksums set currently.

    Ok(checksums)
}

pub fn ffi_uniffi_contract_version(namespace: &initial::Namespace) -> RustFfiFunctionName {
    RustFfiFunctionName(format!(
        "ffi_{}_uniffi_contract_version",
        &namespace.crate_name
    ))
}

pub fn ffi_definitions(namespace: &initial::Namespace) -> Result<Vec<FfiDefinition>> {
    let checksum_defs = checksums(namespace)?.into_iter().map(|checksum| {
        FfiDefinition::RustFunction(FfiFunction {
            name: checksum.fn_name,
            async_data: None,
            arguments: vec![],
            return_type: FfiReturnType {
                ty: Some(FfiType::UInt16),
            },
            has_rust_call_status_arg: false,
            kind: FfiFunctionKind::Checksum,
        })
    });
    let builtin_defs = [FfiDefinition::RustFunction(FfiFunction {
        name: ffi_uniffi_contract_version(namespace),
        async_data: None,
        arguments: vec![],
        return_type: FfiReturnType {
            ty: Some(FfiType::UInt32),
        },
        has_rust_call_status_arg: false,
        kind: FfiFunctionKind::UniffiContractVersion,
    })];
    Ok(builtin_defs.into_iter().chain(checksum_defs).collect())
}
