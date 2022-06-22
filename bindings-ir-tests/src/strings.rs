/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::test_module;
use bindings_ir::ir::*;

pub fn string_tests() -> Module {
    let mut module = test_module();

    module.add_test(
        "test_strings",
        [
            assert_eq(lit::string("aa"), lit::string("aa")),
            assert_ne(lit::string("aa"), lit::string("ab")),
            assert_eq(
                string::concat([
                    lit::string("My values are: ["),
                    lit::int(1),
                    lit::string(", "),
                    lit::int32(2),
                    lit::string(", "),
                    lit::uint8(3),
                    lit::string("]"),
                ]),
                lit::string("My values are: [1, 2, 3]"),
            ),
            assert(ge(
                string::min_byte_len(lit::string("🦊🦊")),
                lit::int("🦊🦊".len() as i32),
            )),
        ],
    );

    module
}
