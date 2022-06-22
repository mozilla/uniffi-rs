/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::test_module;
use bindings_ir::ir::*;


pub fn math_tests() -> Module {
    let mut module = test_module();

    module.add_test(
        "test_integer_math",
        [
            assert_eq(add(int8(), lit::int8(1), lit::int8(1)), lit::int8(2)),
            assert_eq(sub(uint64(), lit::uint64(3), lit::uint64(2)), lit::uint64(1)),
            assert_eq(mul(int32(), lit::int32(2), lit::int32(-2)), lit::int32(-4)),
            assert_eq(div(uint16(), lit::uint16(4), lit::uint16(2)), lit::uint16(2)),
            assert_eq(
                mul(int64(), add(int64(), lit::int64(1), lit::int64(1)), sub(int64(), lit::int64(4), lit::int64(2))),
                lit::int64(4),
            ),
        ],
    );

    module.add_test(
        "test_integer_comparisions",
        [
            assert(gt(lit::int8(3), lit::int8(2))),
            assert(ge(lit::uint16(3), lit::uint16(3))),
            assert(lt(lit::int32(2), lit::int32(3))),
            assert(le(lit::uint32(2), lit::uint32(2))),
            assert_eq(lit::int64(1), lit::int64(1)),
        ],
    );

    module.add_test(
        "test_integer_matching",
        [
            var("arm", string(), lit::string("")),
            match_int(
                int32(),
                lit::int32(2),
                [
                    arm::int(1, [assign("arm", lit::string("one"))]),
                    arm::int(2, [assign("arm", lit::string("two"))]),
                    arm::int_default([assign("arm", lit::string("default"))]),
                ],
            ),
            assert_eq(ident("arm"), lit::string("two")),
            match_int(
                uint8(),
                lit::int32(4),
                [
                    arm::int(1, [assign("arm", lit::string("one"))]),
                    arm::int(2, [assign("arm", lit::string("two"))]),
                    arm::int_default([assign("arm", lit::string("default"))]),
                ],
            ),
            assert_eq(ident("arm"), lit::string("default")),
        ],
    );

    module.add_test(
        "test_boolean_logic",
        [
            assert(lit::boolean(true)),
            assert(not(lit::boolean(false))),
            assert(and(lit::boolean(true), lit::boolean(true))),
            assert(not(and(lit::boolean(true), lit::boolean(false)))),
            assert(or(lit::boolean(true), lit::boolean(false))),
            assert(not(or(lit::boolean(false), lit::boolean(false)))),
        ],
    );

    module
}
