/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::test_module;
use bindings_ir::ir::*;

pub fn basic_tests() -> Module {
    let mut module = test_module();

    module.add_function(FunctionDef {
        vis: public(),
        name: "get_bool".into(),
        throws: None,
        args: vec![],
        return_type: boolean().into(),
        body: block([
            val("foo", boolean(), lit::boolean(true)),
            return_(ident("foo")),
        ]),
    });

    module.add_function(FunctionDef {
        vis: private(),
        name: "invert".into(),
        throws: None,
        args: vec![arg("input", boolean())],
        return_type: boolean().into(),
        body: block([return_(not(ident("input")))]),
    });

    module.add_test(
        "test_function_calls",
        [
            assert(call("get_bool", [])),
            assert(not(call("invert", [call("get_bool", [])]))),
        ],
    );

    module
}
