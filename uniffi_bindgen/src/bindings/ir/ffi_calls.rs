/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::*;
use crate::interface;
use bindings_ir::ir::*;

pub(super) fn build_module(
    module: &mut Module,
    ci: &interface::ComponentInterface,
    cdylib_name: &str,
) {
    // Define FFI Functions
    //
    // Note: we can't use iter_ffi_function_definitions() since those definitions don't track
    // things like mutability or if pointers are consumed or not
    let mut ffi_functions = vec![
        FFIFunctionDef {
            name: ci.ffi_rustbuffer_alloc().name().into(),
            args: vec![arg("size", int32()), arg_call_status()],
            return_type: cstruct("RustBuffer").into(),
        },
        FFIFunctionDef {
            name: ci.ffi_rustbuffer_free().name().into(),
            args: vec![arg("buf", cstruct("RustBuffer")), arg_call_status()],
            return_type: None,
        },
    ];

    for func in ci.function_definitions() {
        ffi_functions.push(func.ffi_func().into_ir());
    }

    for obj in ci.object_definitions() {
        ffi_functions.push(FFIFunctionDef {
            name: obj.ffi_object_free().name().into(),
            args: vec![arg("ptr", pointer(obj.name())), arg_call_status()],
            return_type: None,
        });
        for cons in obj.constructors() {
            ffi_functions.push(cons.ffi_func().into_ir());
        }
        for meth in obj.methods() {
            ffi_functions.push(meth.ffi_func().into_ir());
        }
    }

    // TODO
    // for cb in ci.callback_interface_definitions() {
    // }

    module.add_native_library(cdylib_name, ffi_functions);

    module.add_cstruct(CStructDef {
        name: "RustBuffer".into(),
        fields: vec![
            field("capacity", int32()),
            field("len", int32()),
            field("ptr", nullable(pointer("RustBufferPtr"))),
        ],
    });
    module.add_cstruct(CStructDef {
        name: "RustCallStatus".into(),
        fields: vec![
            field("code", int32()),
            field("error_buf", cstruct("RustBuffer")),
        ],
    });
    module.add_cstruct(CStructDef {
        name: "ForeignBytes".into(),
        fields: vec![
            field("len", int32()),
            field("data", pointer("ForeignBytesPtr")),
        ],
    });
    module.add_function(free_rust_buffer_fn(ci));
    build_error_handlers(module, ci);
}

fn build_error_handlers(module: &mut Module, ci: &interface::ComponentInterface) {
    module.add_function(FunctionDef {
        name: func_names::throw_if_error().into(),
        vis: private(),
        throws: None,
        args: vec![arg("status", cstruct("RustCallStatus"))],
        return_type: None,
        body: block([match_int(
            int32(),
            ident("status.code"),
            [
                arm::int(0, [return_void()]),
                arm::int(
                    1,
                    [
                        // We shouldn't see this status code for functions that don't throw, free
                        // the RustBuffer and raise a generic error.
                        match_nullable(
                            ident("status.error_buf"),
                            arm::some("error_buf", [call_rustbuffer_free(ident("error_buf"))]),
                            arm::null([]),
                        ),
                        raise_internal_exception(lit::string("Unexpected CALL_ERROR")),
                    ],
                ),
                arm::int(
                    2,
                    [
                        // when the rust code sees a panic, it tries to construct a rustbuffer
                        // with the message.  but if that code panics, then it just sends back
                        // an empty buffer.
                        if_else(
                            gt(ident("status.error_buf.len"), lit::int32(0)),
                            [raise_internal_exception(call(
                                "uniffi_lift_string",
                                [ident("status.error_buf")],
                            ))],
                            [raise_internal_exception(lit::string("Rust panic"))],
                        ),
                    ],
                ),
                arm::int_default([raise_internal_exception(string::concat([
                    lit::string("Unkown Rust call status: "),
                    ident("status.code"),
                ]))]),
            ],
        )]),
    });

    for error in ci.error_definitions() {
        let error_type = error.type_();
        module.add_function(FunctionDef {
            name: func_names::throw_if_error_with_throws_type(&error_type).into(),
            vis: private(),
            throws: None,
            args: vec![arg("status", cstruct("RustCallStatus"))],
            return_type: None,
            body: block([match_int(
                int32(),
                ident("status.code"),
                [
                    arm::int(0, [return_void()]),
                    arm::int(1, [raise(error_type.call_lift(ident("status.error_buf")))]),
                    arm::int(
                        2,
                        [
                            // when the rust code sees a panic, it tries to construct a rustbuffer
                            // with the message.  but if that code panics, then it just sends back
                            // an empty buffer.
                            if_else(
                                gt(ident("status.error_buf.len"), lit::int32(0)),
                                [raise_internal_exception(call(
                                    "uniffi_lift_string",
                                    [ident("status.error_buf")],
                                ))],
                                [raise_internal_exception(lit::string("Rust panic"))],
                            ),
                        ],
                    ),
                    arm::int_default([raise_internal_exception(string::concat([
                        lit::string("Unkown Rust call status: "),
                        ident("status.code"),
                    ]))]),
                ],
            )]),
        })
    }
}

fn free_rust_buffer_fn(ci: &interface::ComponentInterface) -> FunctionDef {
    FunctionDef {
        name: "uniffi_rustbuffer_free".into(),
        throws: None,
        vis: private(),
        args: vec![arg("buf", cstruct("RustBuffer"))],
        return_type: None,
        body: block([
            empty_rust_status_var("status"),
            ffi_call(
                ci.ffi_rustbuffer_free().name(),
                [ident("buf"), ref_mut("status")],
            )
            .into_statement(),
            // Note: don't check status, ffi_rustbuffer_free should never fail and if it did
            // there's nothing we can do anyways.
        ]),
    }
}
