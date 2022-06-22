/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::test_module;
use bindings_ir::ir::*;

pub fn compound_tests() -> Module {
    let mut module = test_module();

    module.add_test(
        "test_nulls",
        [
            val("maybe_int", nullable(int32()), some(lit::int32(1))),
            val("maybe_int2", nullable(int32()), lit::null()),
            match_nullable(
                ident("maybe_int"),
                arm::some("x", [assert_eq(ident("x"), lit::int32(1))]),
                arm::null([assert(lit::boolean(false))]),
            ),
            match_nullable(
                ident("maybe_int2"),
                arm::some("x", [assert(lit::boolean(false))]),
                arm::null([]),
            ),
            assert_eq(
                unwrap(ident("maybe_int"), lit::string("unwrap failed")),
                lit::int32(1),
            ),
            assert_raises_with_string(
                lit::string("unwrap failed"),
                unwrap(ident("maybe_int2"), lit::string("unwrap failed")).into_statement(),
            ),
        ],
    );

    module.add_test(
        "test_list_operations",
        [
            mut_val("l", list(int32()), list::create(int32())),
            assert_eq(list::len("l"), lit::int32(0)),
            list::push("l", lit::int32(1)),
            list::push("l", lit::int32(2)),
            list::push("l", lit::int32(3)),
            assert_eq(list::len("l"), lit::int32(3)),
            assert_eq(list::get("l", lit::int32(0)), lit::int32(1)),
            list::set("l", lit::int32(0), lit::int32(4)),
            assert_eq(list::get("l", lit::int32(0)), lit::int32(4)),
            assert_eq(list::pop("l"), lit::int32(3)),
            assert_eq(list::pop("l"), lit::int32(2)),
            assert_eq(list::pop("l"), lit::int32(4)),
            assert_eq(list::len("l"), lit::int32(0)),
        ],
    );

    module.add_test(
        "test_list_equality",
        [
            mut_val("l1", list(int32()), list::create(int32())),
            mut_val("l2", list(int32()), list::create(int32())),
            assert_eq(ident("l1"), ident("l2")),
            list::push("l1", lit::int32(1)),
            assert_ne(ident("l1"), ident("l2")),
            list::push("l2", lit::int32(1)),
            assert_eq(ident("l1"), ident("l2")),
            list::empty("l2"),
            assert_ne(ident("l1"), ident("l2")),
            list::push("l2", lit::int32(2)),
            assert_ne(ident("l1"), ident("l2")),
        ],
    );

    module.add_test(
        "test_list_iteration",
        [
            mut_val("l", list(int32()), list::create(int32())),
            var("sum", int32(), lit::int32(0)),
            list::push("l", lit::int32(1)),
            list::push("l", lit::int32(2)),
            list::push("l", lit::int32(3)),
            list::iterate("l", "i", [add_assign(int32(), "sum", ident("i"))]),
            assert_eq(ident("sum"), lit::int32(6)),
        ],
    );

    module.add_test(
        "test_map_operations",
        [
            mut_val("m", map(int32(), string()), map::create(int32(), string())),
            assert_eq(map::len("m"), lit::int32(0)),
            map::set("m", lit::int32(1), lit::string("apple")),
            map::set("m", lit::int32(2), lit::string("banana")),
            map::set("m", lit::int32(3), lit::string("pear")),
            assert_eq(map::len("m"), lit::int32(3)),
            assert_eq(map::get("m", lit::int32(1)), lit::string("apple")),
            map::set("m", lit::int32(1), lit::string("strawberry")),
            assert_eq(map::get("m", lit::int32(1)), lit::string("strawberry")),
            map::remove("m", lit::int32(1)),
            assert_eq(map::len("m"), lit::int32(2)),
            map::empty("m"),
            assert_eq(map::len("m"), lit::int32(0)),
        ],
    );

    module.add_test(
        "test_map_equality",
        [
            mut_val("m1", map(int32(), string()), map::create(int32(), string())),
            mut_val("m2", map(int32(), string()), map::create(int32(), string())),
            assert_eq(ident("m1"), ident("m2")),
            map::set("m1", lit::int32(1), lit::string("apple")),
            assert_ne(ident("m1"), ident("m2")),
            map::set("m2", lit::int32(1), lit::string("apple")),
            assert_eq(ident("m1"), ident("m2")),
            map::set("m2", lit::int32(1), lit::string("banana")),
            assert_ne(ident("m1"), ident("m2")),
            map::empty("m1"),
            assert_ne(ident("m1"), ident("m2")),
            map::remove("m2", lit::int32(1)),
            assert_eq(ident("m1"), ident("m2")),
        ],
    );

    module.add_test(
        "test_map_iteration",
        [
            mut_val("m", map(int32(), string()), map::create(int32(), string())),
            var("count", int32(), lit::int32(0)),
            map::set("m", lit::int32(1), lit::string("apple")),
            map::set("m", lit::int32(2), lit::string("banana")),
            map::set("m", lit::int32(3), lit::string("strawberry")),
            map::iterate(
                "m",
                "k",
                "v",
                [
                    match_int(
                        int32(),
                        ident("k"),
                        [
                            arm::int(1, [assert_eq(ident("v"), lit::string("apple"))]),
                            arm::int(2, [assert_eq(ident("v"), lit::string("banana"))]),
                            arm::int(3, [assert_eq(ident("v"), lit::string("strawberry"))]),
                            arm::int_default([assert(lit::boolean(false))]),
                        ],
                    ),
                    add_assign(int32(), "count", lit::int32(1)),
                ],
            ),
            assert_eq(ident("count"), lit::int32(3)),
        ],
    );

    module
}
