/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::*;
use crate::interface;
/// Base functionality for FFI Converters
///
/// This module sets up the generic code.  The actual functions are defined in type-specific module
/// (enum_, record, etc.)
use bindings_ir::ir::*;

/// Add lift function definition
pub(super) fn add_lift_func(
    module: &mut Module,
    type_: &interface::Type,
    body: impl IntoIterator<Item = Statement>,
) {
    module.add_function(FunctionDef {
        name: func_names::lift(&type_).into(),
        throws: None,
        vis: private(),
        args: vec![arg("value", type_.ffi_type().ir_lift_type())],
        return_type: Some(type_.clone().into_ir()),
        body: block(body),
    });
}

// Add a lower function definition
pub(super) fn add_lower_func(
    module: &mut Module,
    type_: &interface::Type,
    body: impl IntoIterator<Item = Statement>,
) {
    module.add_function(FunctionDef {
        name: func_names::lower(&type_).into(),
        throws: None,
        vis: private(),
        args: vec![arg("value", type_.clone().into_ir())],
        return_type: Some(type_.ffi_type().ir_lower_type()),
        body: block(body),
    });
}

/// Add an allocation size function definition
pub(super) fn add_allocation_size_func(
    module: &mut Module,
    type_: &interface::Type,
    body: impl IntoIterator<Item = Statement>,
) {
    module.add_function(FunctionDef {
        name: func_names::allocation_size(&type_).into(),
        throws: None,
        vis: private(),
        args: vec![arg("value", type_.clone().into_ir())],
        return_type: int32().into(),
        body: block(body),
    });
}

/// Add a read function definition
pub(super) fn add_read_func(
    module: &mut Module,
    type_: &interface::Type,
    body: impl IntoIterator<Item = Statement>,
) {
    module.add_function(FunctionDef {
        name: func_names::read(&type_).into(),
        throws: None,
        vis: private(),
        args: vec![arg("stream", reference_mut(object("RustBufferStream")))],
        return_type: Some(type_.clone().into_ir()),
        body: block(body),
    });
}

// Add a write function definition
pub(super) fn add_write_func(
    module: &mut Module,
    type_: &interface::Type,
    body: impl IntoIterator<Item = Statement>,
) {
    module.add_function(FunctionDef {
        name: func_names::write(&type_).into(),
        throws: None,
        vis: private(),
        args: vec![
            arg("stream", reference_mut(object("RustBufferStream"))),
            arg("value", type_.clone().into_ir()),
        ],
        return_type: None,
        body: block(body),
    });
}

// Add lift and lower functions that leverage the read/write function when the FFI type is
// RustBuffer
pub(super) fn add_rust_buffer_lift_and_lower_funcs(
    module: &mut Module,
    ci: &interface::ComponentInterface,
    type_: &interface::Type,
) {
    add_lower_func(
        module,
        type_,
        [
            empty_rust_status_var("status"),
            destructure(
                "RustBuffer",
                ffi_call(
                    ci.ffi_rustbuffer_alloc().name(),
                    [
                        cast::int32(type_.call_allocation_size(ident("value"))),
                        ref_mut("status"),
                    ],
                ),
                ["capacity", "len", "ptr_maybe_null"],
            ),
            call_throw_if_error("status"),
            val(
                "ptr",
                pointer("RustBufferPtr"),
                unwrap(
                    ident("ptr_maybe_null"),
                    lit::string(format!(
                        "{}.lower: ffi_rustbuffer_alloc returned null pointer",
                        func_names::lower(type_)
                    )),
                ),
            ),
            // Throw if the pointer is null
            var(
                "stream",
                object("RustBufferStream"),
                buf::create("RustBufferStream", ident("ptr"), ident("len")),
            ),
            type_
                .call_write(ref_mut("stream"), ident("value"))
                .into_statement(),
            return_(create_cstruct(
                "RustBuffer",
                [
                    ident("capacity"),
                    // The new length is the position of the buffer stream after writing
                    buf::pos(ident("stream")),
                    buf::into_ptr("RustBufferPtr", ident("stream")),
                ],
            )),
        ],
    );
    add_lift_func(
        module,
        type_,
        [
            destructure(
                "RustBuffer",
                ident("value"),
                ["capacity", "len", "ptr_maybe_null"],
            ),
            val(
                "ptr",
                pointer("RustBufferPtr"),
                unwrap(
                    ident("ptr_maybe_null"),
                    lit::string(format!(
                        "{}.lift called with null RustBuffer pointer",
                        func_names::lower(type_)
                    )),
                ),
            ),
            var(
                "stream",
                object("RustBufferStream"),
                buf::create("RustBufferStream", ident("ptr"), ident("len")),
            ),
            val(
                "return_value",
                type_.clone().into_ir(),
                type_.call_read(ref_mut("stream")),
            ),
            call_rustbuffer_free_from_components(ident("capacity"), ident("len"), ident("ptr")),
            return_(ident("return_value")),
        ],
    )
}
