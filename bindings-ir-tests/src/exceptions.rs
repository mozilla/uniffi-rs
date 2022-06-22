/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::test_module;
use bindings_ir::ir::*;

pub fn exception_tests() -> Module {
    let mut module = test_module();

    module.add_exception_base(exception_base_def("ExceptionBase"));
    module.add_exception_base(exception_base_def_child("ExceptionBase", "ExceptionMiddle"));
    module.add_exception(ExceptionDef {
        name: "TestException".into(),
        parent: "ExceptionMiddle".into(),
        ..ExceptionDef::default()
    });
    module.add_exception(ExceptionDef {
        name: "ExceptionWithFields".into(),
        parent: "ExceptionMiddle".into(),
        fields: vec![field("x", int32())],
        ..ExceptionDef::default()
    });
    module.add_exception(ExceptionDef {
        name: "ExceptionWithAsString".into(),
        parent: "ExceptionMiddle".into(),
        fields: vec![field("reason", string())],
        as_string: AsStringMethod {
            body: block([return_(string::concat([
                lit::string("error: "),
                ident("reason"),
            ]))]),
        }
        .into(),
    });

    module.add_function(FunctionDef {
        vis: private(),
        name: "throw_test_exception".into(),
        throws: Some("ExceptionBase".into()),
        args: vec![],
        return_type: None,
        body: block([raise(create_exception("TestException", []))]),
    });

    module.add_test(
        "test_exceptions",
        [
            assert_raises(
                "TestException",
                call("throw_test_exception", []).into_statement(),
            ),
            assert_raises(
                "ExceptionMiddle",
                call("throw_test_exception", []).into_statement(),
            ),
            assert_raises(
                "ExceptionBase",
                call("throw_test_exception", []).into_statement(),
            ),
            assert_raises(
                "ExceptionWithFields",
                raise(create_exception("ExceptionWithFields", [lit::int(2)])),
            ),
            assert_raises_with_string(
                lit::string("error: bad value"),
                raise(create_exception(
                    "ExceptionWithAsString",
                    [lit::string("bad value")],
                )),
            ),
        ],
    );

    module.add_test(
        "test_runtime_exceptions",
        [assert_raises_with_string(
            lit::string("something failed"),
            raise_internal_exception(string::concat([
                lit::string("something "),
                lit::string("failed"),
            ])),
        )],
    );

    module.add_function(FunctionDef {
        name: "check_is_instance".into(),
        vis: private(),
        args: vec![arg("exc", object("ExceptionBase"))],
        return_type: boolean().into(),
        throws: None,
        body: block([return_(is_instance(ident("exc"), "TestException"))]),
    });

    module.add_test(
        "test_is_instance",
        [
            assert(call(
                "check_is_instance",
                [create_exception("TestException", [])],
            )),
            assert(not(call(
                "check_is_instance",
                [create_exception("ExceptionWithFields", [lit::int(2)])],
            ))),
        ],
    );

    module
}
