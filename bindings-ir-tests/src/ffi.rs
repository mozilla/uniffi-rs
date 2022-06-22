/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::test_module;
use bindings_ir::ir::*;

pub fn ffi_tests() -> Module {
    let mut module = test_module();

    module.add_test(
        "test_call_roundtrip_functions",
        [
            ("roundtrip_u8", lit::uint8(1)),
            ("roundtrip_i8", lit::int8(-1)),
            ("roundtrip_u16", lit::uint16(2)),
            ("roundtrip_i16", lit::int16(-2)),
            ("roundtrip_u32", lit::uint32(4)),
            ("roundtrip_i32", lit::int32(-4)),
            ("roundtrip_u64", lit::uint64(8)),
            ("roundtrip_i64", lit::int64(-8)),
            ("roundtrip_f32", lit::float32("0.5")),
            ("roundtrip_f64", lit::float64("0.5")),
        ]
        .into_iter()
        .map(|(ffi_func, literal)| assert_eq(ffi_call(ffi_func, [literal.clone()]), literal)),
    );

    module.add_test(
        "test_cast",
        [
            assert_eq(cast::uint8(lit::int(1)), lit::uint8(1)),
            assert_eq(cast::int8(lit::int(-1)), lit::int8(-1)),
            assert_eq(cast::uint16(lit::int(2)), lit::uint16(2)),
            assert_eq(cast::int16(lit::int(-2)), lit::int16(-2)),
            assert_eq(cast::uint32(lit::int(4)), lit::uint32(4)),
            assert_eq(cast::int32(lit::int(-4)), lit::int32(-4)),
            assert_eq(cast::uint64(lit::int(8)), lit::uint64(8)),
            assert_eq(cast::int64(lit::int(-8)), lit::int64(-8)),
        ],
    );

    module.add_test(
        "test_pointer_size",
        [assert_eq(
            pointer_size(),
            lit::int(std::mem::size_of::<*const u8>() as i32),
        )],
    );

    module
}
