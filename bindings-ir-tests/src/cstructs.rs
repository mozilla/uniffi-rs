/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::test_module;
use bindings_ir::ir::*;

pub fn cstructs_tests() -> Module {
    let mut module = test_module();

    module.add_test(
        "test_structs",
        [
            mut_val(
                "n",
                cstruct("Numbers"),
                ffi_call("make_numbers", [lit::int32(2), lit::int32(3)]),
            ),
            assert_eq(ident("n.a"), lit::int32(2)),
            assert_eq(ident("n.b"), lit::int32(3)),
            val(
                "n2",
                cstruct("Numbers"),
                create_cstruct("Numbers", [lit::int32(2), lit::int32(3)]),
            ),
            assert_eq(ident("n.a"), lit::int32(2)),
            assert_eq(ident("n.b"), lit::int32(3)),
            assert_eq(ffi_call("add_numbers", [ident("n")]), lit::int32(5)),
            assert_eq(ffi_call("add_numbers", [ident("n2")]), lit::int32(5)),
            set(ident("n"), "a", lit::int32(30)),
            assert_eq(ffi_call("add_numbers", [ident("n")]), lit::int32(33)),
            destructure("Numbers", ident("n"), ["a", "b"]),
            assert_eq(ident("a"), lit::int32(30)),
            assert_eq(ident("b"), lit::int32(3)),
        ],
    );

    module.add_test(
        "test_cstruct_field_types",
        [
            var(
                "x",
                cstruct("CStructWithAllTypes"),
                create_cstruct(
                    "CStructWithAllTypes",
                    [
                        lit::int8(-1),
                        lit::uint8(1),
                        lit::int16(-2),
                        lit::uint16(2),
                        lit::int32(-3),
                        lit::uint32(3),
                        lit::int64(-4),
                        lit::uint64(4),
                        lit::float32("1.0"),
                        lit::float64("-3.0"),
                        create_cstruct("Numbers", [lit::int(0), lit::int(1)]),
                        ffi_call("get_read_buffer_ptr", []),
                    ],
                ),
            ),
            set(ident("x"), "i8", lit::int8(0)),
            set(ident("x"), "u8", lit::uint8(0)),
            set(ident("x"), "i16", lit::int16(0)),
            set(ident("x"), "u16", lit::uint16(0)),
            set(ident("x"), "i32", lit::int32(0)),
            set(ident("x"), "u32", lit::uint32(0)),
            set(ident("x"), "i64", lit::int64(0)),
            set(ident("x"), "u64", lit::uint64(0)),
            set(ident("x"), "f32", lit::float32("0.5")),
            set(ident("x"), "f64", lit::float64("-1.2")),
            set(
                ident("x"),
                "numbers",
                create_cstruct("Numbers", [lit::int(2), lit::int(3)]),
            ),
            set(ident("x"), "pointer", ffi_call("get_write_buffer_ptr", [])),
            set(ident("x"), "nullable_pointer", lit::null()),
        ],
    );

    module.add_test(
        "test_struct_refs",
        [
            mut_val(
                "n",
                cstruct("Numbers"),
                create_cstruct("Numbers", [lit::int32(2), lit::int32(3)]),
            ),
            assert_eq(ffi_call("add_numbers_ref", [ref_("n")]), lit::int32(5)),
            ffi_call("double_each_number", [ref_mut("n")]).into_statement(),
            assert_eq(ident("n.a"), lit::int32(4)),
            assert_eq(ident("n.b"), lit::int32(6)),
        ],
    );

    module
}
