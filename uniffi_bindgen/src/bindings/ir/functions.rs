/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::*;
use crate::interface;
use bindings_ir::ir::*;
use std::iter::once;

pub(super) fn build_module(module: &mut Module, ci: &interface::ComponentInterface) {
    for func in ci.function_definitions() {
        let make_ffi_call = ffi_call(
            func.ffi_func().name(),
            func.arguments()
                .into_iter()
                .map(|a| a.type_().call_lower(ident(a.name())))
                .chain(once(ref_mut("uniffi_call_status"))),
        );

        // Create statements to make the FFI call and return the result on success
        let (ffi_call_stmt, success_return) = match func.return_type() {
            None => (make_ffi_call.into_statement(), return_void()),
            Some(return_type) => (
                val(
                    "uniffi_call_return",
                    return_type.clone().into_ir(),
                    return_type.call_lift(make_ffi_call),
                ),
                return_(ident("uniffi_call_return")),
            ),
        };

        let body = vec![
            var(
                "uniffi_call_status",
                cstruct("RustCallStatus"),
                create_cstruct(
                    "RustCallStatus",
                    [
                        lit::int32(0),
                        create_cstruct("RustBuffer", [lit::int32(0), lit::int32(0), lit::null()]),
                    ],
                ),
            ),
            ffi_call_stmt,
            // Throw an error if `uniffi_call_status` indicates we should
            match func.throws_type() {
                Some(error_type) => {
                    call_throw_if_error_with_type(&error_type, "uniffi_call_status")
                }
                None => call_throw_if_error("uniffi_call_status"),
            },
            // If not, return success
            success_return,
        ];

        module.add_function(FunctionDef {
            vis: public(),
            name: func.name().into(),
            throws: func.throws_name().map(ClassName::from),
            args: func.arguments().into_ir(),
            return_type: func.return_type().into_ir(),
            body: block(body),
        })
    }
}
