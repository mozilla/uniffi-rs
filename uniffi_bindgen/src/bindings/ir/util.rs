/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::func_names;
use crate::interface;
use bindings_ir::ir::*;

pub(super) fn empty_rust_status_var(name: &str) -> Statement {
    var(
        name,
        cstruct("RustCallStatus"),
        create_cstruct(
            "RustCallStatus",
            [
                lit::int32(0),
                create_cstruct("RustBuffer", [lit::int32(0), lit::int32(0), lit::null()]),
            ],
        ),
    )
}

pub(super) fn arg_call_status() -> Argument {
    arg(
        "uniffi_out_status",
        reference_mut(cstruct("RustCallStatus")),
    )
}

pub(super) fn call_rustbuffer_free(buf: Expression) -> Statement {
    call("uniffi_rustbuffer_free", [buf]).into_statement()
}

pub(super) fn call_rustbuffer_free_from_components(
    capacity: Expression,
    len: Expression,
    ptr: Expression,
) -> Statement {
    call_rustbuffer_free(create_cstruct("RustBuffer", [capacity, len, ptr]))
}

pub(super) fn call_throw_if_error(status_var: &str) -> Statement {
    call(func_names::throw_if_error(), [ident(status_var)]).into_statement()
}

pub(super) fn call_throw_if_error_with_type(
    error_type: &interface::Type,
    status_var: &str,
) -> Statement {
    call(
        func_names::throw_if_error_with_throws_type(error_type),
        [ident(status_var)],
    )
    .into_statement()
}
