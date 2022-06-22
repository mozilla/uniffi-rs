/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::*;
use crate::interface;
use bindings_ir::ir::*;

pub(super) fn build_module(module: &mut Module, ci: &interface::ComponentInterface) {
    for type_ in ci.iter_types() {
        match type_ {
            interface::Type::Optional(inner) => {
                add_optional_ffi_converter(module, ci, type_, inner)
            }
            interface::Type::Sequence(inner) => {
                unimplemented!()
            }
            interface::Type::Map(key, value) => {
                unimplemented!()
            }
            _ => (),
        }
    }
}

fn add_optional_ffi_converter(
    module: &mut Module,
    ci: &interface::ComponentInterface,
    type_: &interface::Type,
    inner: &interface::Type,
) {
    add_allocation_size_func(
        module,
        &type_,
        [match_nullable(
            ident("value"),
            arm::some(
                "inner_value",
                [return_(add(
                    int32(),
                    lit::int32(1),
                    inner.call_allocation_size(ident("inner_value")),
                ))],
            ),
            arm::null([return_(lit::int32(1))]),
        )],
    );
    add_read_func(
        module,
        &type_,
        [if_else(
            eq(lit::int8(1), buf::read_int8(ident("stream"))),
            [return_(inner.call_read(ident("stream")))],
            [return_(lit::null())],
        )],
    );
    add_write_func(
        module,
        &type_,
        [match_nullable(
            ident("value"),
            arm::some(
                "inner_value",
                [
                    buf::write_int8(ident("stream"), lit::int8(1)),
                    inner
                        .call_write(ident("stream"), ident("inner_value"))
                        .into_statement(),
                ],
            ),
            arm::null([buf::write_int8(ident("stream"), lit::int8(0))]),
        )],
    );
    add_rust_buffer_lift_and_lower_funcs(module, ci, type_);
}
