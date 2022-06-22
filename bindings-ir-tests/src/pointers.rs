/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::test_module;
use bindings_ir::ir::*;

pub fn pointer_tests() -> Module {
    let mut module = test_module();

    module.add_test(
        "test_pointer_args_and_returns",
        [
            val(
                "ptr",
                pointer("BufPtr"),
                ffi_call("get_read_buffer_ptr", []),
            ),
            assert_eq(
                ffi_call("is_read_buffer_ptr", [ident("ptr")]),
                lit::uint8(1),
            ),
        ],
    );

    module.add_test(
        "test_buffer_read",
        [
            val(
                "read_ptr",
                pointer("BufPtr"),
                ffi_call("get_read_buffer_ptr", []),
            ),
            mut_val(
                "read_buf",
                object("BufStream"),
                buf::create(
                    "BufStream",
                    ident("read_ptr"),
                    ffi_call("get_read_buffer_size", []),
                ),
            ),
            assert_eq(
                buf::size(ident("read_buf")),
                ffi_call("get_read_buffer_size", []),
            ),
            assert_eq(buf::read_uint8(ident("read_buf")), lit::uint8(1)),
            assert_eq(buf::pos(ident("read_buf")), lit::int(1)),
            assert_eq(buf::read_int8(ident("read_buf")), lit::int8(-1)),
            assert_eq(buf::pos(ident("read_buf")), lit::int(2)),
            assert_eq(buf::read_uint16(ident("read_buf")), lit::uint16(2)),
            assert_eq(buf::pos(ident("read_buf")), lit::int(4)),
            assert_eq(buf::read_int16(ident("read_buf")), lit::int16(-2)),
            assert_eq(buf::pos(ident("read_buf")), lit::int(6)),
            assert_eq(buf::read_uint32(ident("read_buf")), lit::uint32(4)),
            assert_eq(buf::pos(ident("read_buf")), lit::int(10)),
            assert_eq(buf::read_int32(ident("read_buf")), lit::int32(-4)),
            assert_eq(buf::pos(ident("read_buf")), lit::int(14)),
            assert_eq(buf::read_uint64(ident("read_buf")), lit::uint64(8)),
            assert_eq(buf::pos(ident("read_buf")), lit::int(22)),
            assert_eq(buf::read_int64(ident("read_buf")), lit::int64(-8)),
            assert_eq(buf::pos(ident("read_buf")), lit::int(30)),
            assert_eq(buf::read_float32(ident("read_buf")), lit::float32("1.5")),
            assert_eq(buf::pos(ident("read_buf")), lit::int(34)),
            assert_eq(buf::read_float64(ident("read_buf")), lit::float64("-0.5")),
            assert_eq(buf::pos(ident("read_buf")), lit::int(42)),
            assert_eq(
                buf::read_pointer("BufPtr", ident("read_buf")),
                ident("read_ptr"),
            ),
            assert_eq(buf::pos(ident("read_buf")), lit::int(50)),
            assert_eq(
                buf::into_ptr("BufPtr", ident("read_buf")),
                ffi_call("get_read_buffer_ptr", []),
            ),
            buf::set_pos(ident("read_buf"), lit::int(2)),
            assert_eq(buf::read_uint16(ident("read_buf")), lit::uint16(2)),
        ],
    );

    module.add_test(
        "test_buffer_write",
        [
            mut_val(
                "write_buf",
                object("BufStream"),
                buf::create(
                    "BufStream",
                    ffi_call("get_write_buffer_ptr", []),
                    ffi_call("get_write_buffer_size", []),
                ),
            ),
            buf::write_uint8(ident("write_buf"), lit::uint8(1)),
            assert_eq(buf::pos(ident("write_buf")), lit::int(1)),
            buf::write_int8(ident("write_buf"), lit::int8(-1)),
            assert_eq(buf::pos(ident("write_buf")), lit::int(2)),
            buf::write_uint16(ident("write_buf"), lit::uint16(2)),
            assert_eq(buf::pos(ident("write_buf")), lit::int(4)),
            buf::write_int16(ident("write_buf"), lit::int16(-2)),
            assert_eq(buf::pos(ident("write_buf")), lit::int(6)),
            buf::write_uint32(ident("write_buf"), lit::uint32(4)),
            assert_eq(buf::pos(ident("write_buf")), lit::int(10)),
            buf::write_int32(ident("write_buf"), lit::int32(-4)),
            assert_eq(buf::pos(ident("write_buf")), lit::int(14)),
            buf::write_uint64(ident("write_buf"), lit::uint64(8)),
            assert_eq(buf::pos(ident("write_buf")), lit::int(22)),
            buf::write_int64(ident("write_buf"), lit::int64(-8)),
            assert_eq(buf::pos(ident("write_buf")), lit::int(30)),
            buf::write_float32(ident("write_buf"), lit::float32("1.5")),
            assert_eq(buf::pos(ident("write_buf")), lit::int(34)),
            buf::write_float64(ident("write_buf"), lit::float64("-0.5")),
            assert_eq(buf::pos(ident("write_buf")), lit::int(42)),
            buf::write_pointer(
                "BufPtr",
                ident("write_buf"),
                ffi_call("get_read_buffer_ptr", []),
            ),
            assert_eq(buf::pos(ident("write_buf")), lit::int(50)),
            assert_eq(
                ffi_call("write_buffer_matches_read_buffer", []),
                lit::uint8(1),
            ),
        ],
    );

    module.add_test(
        "test_buffer_read_string",
        [
            val("size", int32(), ffi_call("get_string_buffer_size", [])),
            mut_val(
                "string_buf",
                object("BufStream"),
                buf::create(
                    "BufStream",
                    ffi_call("get_string_buffer_ptr", []),
                    ident("size"),
                ),
            ),
            assert_eq(
                buf::read_string(ident("string_buf"), ident("size")),
                lit::string("🦊test-string🦊"),
            ),
            assert_eq(buf::pos(ident("string_buf")), ident("size")),
        ],
    );

    module.add_test(
        "test_buffer_write_string",
        [
            mut_val(
                "write_string_buf",
                object("BufStream"),
                buf::create(
                    "BufStream",
                    ffi_call("get_write_buffer_ptr", []),
                    ffi_call("get_write_buffer_size", []),
                ),
            ),
            buf::write_string(ident("write_string_buf"), lit::string("🦊test-string🦊")),
            assert_eq(
                ffi_call("write_buffer_matches_string_buffer", []),
                lit::uint8(1),
            ),
            assert_eq(
                buf::pos(ident("write_string_buf")),
                ffi_call("get_string_buffer_size", []),
            ),
        ],
    );

    module
}
