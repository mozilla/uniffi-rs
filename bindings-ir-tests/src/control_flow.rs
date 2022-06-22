/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::test_module;
use bindings_ir::ir::*;

pub fn control_flow_tests() -> Module {
    let mut module = test_module();

    module.add_function(FunctionDef {
        vis: private(),
        name: "if_checker".into(),
        throws: None,
        args: vec![arg("x", boolean()), arg("y", boolean())],
        return_type: string().into(),
        body: block([
            if_(ident("x"), [return_(lit::string("x is true"))]),
            if_else(
                ident("y"),
                [return_(lit::string("y is true"))],
                [return_(lit::string("neither is true"))],
            ),
        ]),
    });

    module.add_test(
        "test_if",
        [
            assert_eq(
                lit::string("x is true"),
                call("if_checker", [lit::boolean(true), lit::boolean(false)]),
            ),
            assert_eq(
                lit::string("y is true"),
                call("if_checker", [lit::boolean(false), lit::boolean(true)]),
            ),
            assert_eq(
                lit::string("neither is true"),
                call("if_checker", [lit::boolean(false), lit::boolean(false)]),
            ),
        ],
    );

    module.add_test(
        "test_for",
        [
            mut_val("l", list(int32()), list::create(int32())),
            for_(
                "x",
                lit::int32(0),
                lit::int32(3),
                [list::push("l", mul(int32(), ident("x"), ident("x")))],
            ),
            assert_eq(list::len("l"), lit::int32(3)),
            assert_eq(list::get("l", lit::int32(0)), lit::int32(0)),
            assert_eq(list::get("l", lit::int32(1)), lit::int32(1)),
            assert_eq(list::get("l", lit::int32(2)), lit::int32(4)),
        ],
    );

    module.add_test(
        "test_loop_and_break",
        [
            mut_val("l", list(int32()), list::create(int32())),
            var("i", int32(), lit::int32(0)),
            loop_([
                list::push("l", ident("i")),
                if_(gt(ident("i"), lit::int32(2)), [break_()]),
                add_assign(int32(), "i", lit::int32(1)),
            ]),
            assert_eq(list::len("l"), lit::int32(4)),
            assert_eq(list::get("l", lit::int32(0)), lit::int32(0)),
            assert_eq(list::get("l", lit::int32(1)), lit::int32(1)),
            assert_eq(list::get("l", lit::int32(2)), lit::int32(2)),
            assert_eq(list::get("l", lit::int32(3)), lit::int32(3)),
        ],
    );

    module
}
