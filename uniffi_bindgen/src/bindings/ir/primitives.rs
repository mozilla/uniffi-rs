/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::*;
use crate::interface;
use bindings_ir::ir::*;

pub(super) fn build_module(module: &mut Module, ci: &ComponentInterface) {
    for type_ in ci.iter_types() {
        match type_ {
            interface::Type::Int8 => {
                add_simple_ffi_converters(module, type_, buf::read_int8, buf::write_int8, 1)
            }
            interface::Type::UInt8 => {
                add_simple_ffi_converters(module, type_, buf::read_uint8, buf::write_uint8, 1)
            }
            interface::Type::Int16 => {
                add_simple_ffi_converters(module, type_, buf::read_int16, buf::write_int16, 2)
            }
            interface::Type::UInt16 => {
                add_simple_ffi_converters(module, type_, buf::read_uint16, buf::write_uint16, 2)
            }
            interface::Type::Int32 => {
                add_simple_ffi_converters(module, type_, buf::read_int32, buf::write_int32, 4)
            }
            interface::Type::UInt32 => {
                add_simple_ffi_converters(module, type_, buf::read_uint32, buf::write_uint32, 4)
            }
            interface::Type::Int64 => {
                add_simple_ffi_converters(module, type_, buf::read_int64, buf::write_int64, 8)
            }
            interface::Type::UInt64 => {
                add_simple_ffi_converters(module, type_, buf::read_uint64, buf::write_uint64, 8)
            }
            interface::Type::Float32 => {
                add_simple_ffi_converters(module, type_, buf::read_float32, buf::write_float32, 4)
            }
            interface::Type::Float64 => {
                add_simple_ffi_converters(module, type_, buf::read_float64, buf::write_float64, 8)
            }
            interface::Type::Boolean => add_boolean_ffi_converters(module),
            _ => (),
        }
    }
    // Unconditionally add the string FFI converters, since we use them for error handling.
    add_string_ffi_converters(module, ci);
}

fn add_simple_ffi_converters(
    module: &mut Module,
    type_: &interface::Type,
    read_fn: impl Fn(Expression) -> Expression,
    write_fn: impl Fn(Expression, Expression) -> Statement,
    allocation_size: i32,
) {
    add_lift_func(module, &type_, [return_(ident("value"))]);
    add_lower_func(module, &type_, [return_(ident("value"))]);
    add_allocation_size_func(module, &type_, [return_(lit::int32(allocation_size))]);
    add_read_func(module, &type_, [return_(read_fn(ident("stream")))]);
    add_write_func(module, &type_, [write_fn(ident("stream"), ident("value"))]);
}

// Handle booleans by converting to/from a u8
fn add_boolean_ffi_converters(module: &mut Module) {
    let type_ = interface::Type::Boolean;
    add_lift_func(module, &type_, [return_(eq(lit::int8(1), ident("value")))]);
    add_lower_func(
        module,
        &type_,
        [if_else(
            ident("value"),
            [return_(lit::int8(1))],
            [return_(lit::int8(0))],
        )],
    );
    add_allocation_size_func(module, &type_, [return_(lit::int32(1))]);
    add_read_func(
        module,
        &type_,
        [return_(type_.call_lift(buf::read_int8(ident("stream"))))],
    );
    add_write_func(
        module,
        &type_,
        [buf::write_int8(
            ident("stream"),
            type_.call_lower(ident("value")),
        )],
    );
}

// Handle booleans by converting to/from a u8
fn add_string_ffi_converters(module: &mut Module, ci: &ComponentInterface) {
    let type_ = interface::Type::String;
    add_allocation_size_func(
        module,
        &type_,
        [return_(add(
            int32(),
            // Size needed for the length data
            lit::int32(4),
            // Size needed for the string
            string::min_byte_len(ident("value")),
        ))],
    );
    add_read_func(
        module,
        &type_,
        [
            val("size", int32(), buf::read_int32(ident("stream"))),
            return_(buf::read_string(ident("stream"), ident("size"))),
        ],
    );
    add_write_func(
        module,
        &type_,
        [
            // Write a placeholder for the string size
            val("start_pos", int32(), buf::pos(ident("stream"))),
            buf::write_int32(ident("stream"), lit::int32(0)),
            // Write the string, noting the size
            buf::write_string(ident("stream"), ident("value")),
            // Go back, write the correct string size, then reposition
            val("current_pos", int32(), buf::pos(ident("stream"))),
            buf::set_pos(ident("stream"), ident("start_pos")),
            buf::write_int32(
                ident("stream"),
                sub(
                    int32(),
                    ident("current_pos"),
                    add(int32(), ident("start_pos"), lit::int32(4)),
                ),
            ),
            buf::set_pos(ident("stream"), ident("current_pos")),
        ],
    );
    // We could almost use add_rust_buffer_lift_and_lower_funcs here, however there's a subtle
    // difference.  When lifting/lowering strings directly, we don't write the size of the string
    // to the buffer since it's redundant with the `len` field of the buffer
    add_lower_func(
        module,
        &type_,
        [
            empty_rust_status_var("status"),
            destructure(
                "RustBuffer",
                ffi_call(
                    ci.ffi_rustbuffer_alloc().name(),
                    [
                        type_.call_allocation_size(ident("value")),
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
                    lit::string("String.lower: ffi_rustbuffer_alloc returned null pointer"),
                ),
            ),
            // Throw if the pointer is null
            var(
                "stream",
                object("RustBufferStream"),
                buf::create("RustBufferStream", ident("ptr"), ident("len")),
            ),
            buf::write_string(ident("stream"), ident("value")),
            return_(create_cstruct(
                "RustBuffer",
                [
                    ident("capacity"),
                    buf::pos(ident("stream")),
                    buf::into_ptr("RustBufferPtr", ident("stream")),
                ],
            )),
        ],
    );
    add_lift_func(
        module,
        &type_,
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
                    lit::string("String.lift called with null RustBuffer pointer"),
                ),
            ),
            var(
                "stream",
                object("RustBufferStream"),
                buf::create("RustBufferStream", ident("ptr"), ident("len")),
            ),
            val(
                "return_value",
                string(),
                buf::read_string(ident("stream"), ident("len")),
            ),
            call_rustbuffer_free_from_components(ident("capacity"), ident("len"), ident("ptr")),
            return_(ident("return_value")),
        ],
    );
}
