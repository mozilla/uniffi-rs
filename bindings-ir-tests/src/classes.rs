/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::test_module;
use bindings_ir::ir::*;

pub fn class_tests() -> Module {
    let mut module = test_module();

    module.add_class(ClassDef {
        vis: private(),
        name: "Adder".into(),
        fields: vec![field("ptr", pointer("Adder"))],
        constructor: Constructor {
            vis: public(),
            args: vec![arg("amount", int32())],
            initializers: vec![ffi_call("adder_create", [ident("amount")])],
        }
        .into(),
        methods: vec![
            Method {
                vis: public(),
                method_type: MethodType::Normal,
                throws: None,
                name: "add".into(),
                args: vec![arg("x", int32())],
                return_type: int32().into(),
                body: block([return_(ffi_call(
                    "adder_add",
                    [get(this(), "ptr"), ident("x")],
                ))]),
            },
            Method {
                vis: public(),
                method_type: MethodType::Static,
                throws: None,
                name: "static_add".into(),
                args: vec![arg("x", int32()), arg("y", int32())],
                return_type: int32().into(),
                body: block([return_(add(int32(), ident("x"), ident("y")))]),
            },
        ],
        destructor: Destructor {
            body: block([ffi_call("adder_destroy", [ident("ptr")]).into_statement()]),
        }
        .into(),
        into_rust: IntoRustMethod {
            return_type: pointer("Adder"),
            body: block([return_(ident("ptr"))]),
        }
        .into(),
    });

    module.add_test(
        "test_adder",
        [
            val(
                "adder",
                object("Adder"),
                create_class("Adder", [lit::int32(5)]),
            ),
            assert_eq(
                method_call(ident("adder"), "add", [lit::int32(5)]),
                lit::int32(10),
            ),
            assert_eq(
                method_call(ident("adder"), "add", [lit::int32(10)]),
                lit::int32(15),
            ),
            assert_eq(
                static_method_call("Adder", "static_add", [lit::int32(2), lit::int32(3)]),
                lit::int32(5),
            ),
        ],
    );

    // Test using a reference to an object
    module.add_function(FunctionDef {
        vis: private(),
        name: "use_adder".into(),
        throws: None,
        args: vec![arg("adder", reference(object("Adder")))],
        return_type: int32().into(),
        body: block([return_(method_call(ident("adder"), "add", [lit::int32(1)]))]),
    });
    module.add_test(
        "test_adder_ref",
        [
            val(
                "adder",
                object("Adder"),
                create_class("Adder", [lit::int32(1)]),
            ),
            assert_eq(call("use_adder", [ref_("adder")]), lit::int32(2)),
        ],
    );

    // Create an adder, then drop the reference so that we can test destructors
    module.add_function(FunctionDef {
        vis: private(),
        name: "create_adder_then_quit".into(),
        throws: None,
        args: vec![],
        return_type: None,
        body: block([create_class("Adder", [lit::int32(5)]).into_statement()]),
    });

    // Create an adder, then drop the reference so that we can test destructors
    module.add_function(FunctionDef {
        vis: private(),
        name: "create_adder_consume_then_quit".into(),
        throws: None,
        args: vec![],
        return_type: None,
        body: block([ffi_call(
            "adder_consume",
            [into_rust(create_class("Adder", [lit::int32(5)]))],
        )
        .into_statement()]),
    });

    module.add_test(
        "test_adder_destructor",
        [
            gc(),
            ffi_call("adder_reset_free_count", []).into_statement(),
            call("create_adder_then_quit", []).into_statement(),
            gc(),
            assert_eq(ffi_call("adder_get_free_count", []), lit::uint32(1)),
        ],
    );

    module.add_test(
        "test_adder_consume",
        [
            gc(),
            ffi_call("adder_reset_free_count", []).into_statement(),
            call("create_adder_consume_then_quit", []).into_statement(),
            gc(),
            assert_eq(ffi_call("adder_get_free_count", []), lit::uint32(0)),
        ],
    );

    // Data class tests
    module.add_data_class(DataClassDef {
        vis: public(),
        name: "TestDataClass".into(),
        fields: vec![field("adder", object("Adder")), mut_field("value", int32())],
    });

    module.add_test(
        "test_data_class",
        [
            val(
                "test_data_class",
                object("TestDataClass"),
                create_data_class(
                    "TestDataClass",
                    [create_class("Adder", [lit::int32(5)]), lit::int32(10)],
                ),
            ),
            assert_eq(
                method_call(
                    ident("test_data_class.adder"),
                    "add",
                    [ident("test_data_class.value")],
                ),
                lit::int32(15),
            ),
        ],
    );

    module.add_function(FunctionDef {
        vis: private(),
        name: "create_test_data_class_then_quit".into(),
        throws: None,
        args: vec![],
        return_type: None,
        body: block([create_class("Adder", [lit::int32(5)]).into_statement()]),
    });

    module.add_test(
        "test_data_class_member_destructor",
        [
            gc(),
            ffi_call("adder_reset_free_count", []).into_statement(),
            call("create_test_data_class_then_quit", []).into_statement(),
            gc(),
            assert_eq(ffi_call("adder_get_free_count", []), lit::uint32(1)),
        ],
    );

    // Data class tests
    module.add_class(ClassDef {
        vis: private(),
        name: "MutableClass".into(),
        fields: vec![mut_field("value", int32())],
        constructor: Constructor {
            vis: public(),
            args: vec![],
            initializers: vec![lit::int32(0)],
        }
        .into(),
        methods: vec![Method {
            vis: private(),
            method_type: MethodType::Mutable,
            throws: None,
            name: "inc_value".into(),
            args: vec![],
            return_type: None,
            body: block([set(this(), "value", add(int32(), lit::int32(1), get(this(), "value")))]),
        }],
        ..ClassDef::default()
    });

    // Test using a reference to an object
    module.add_function(FunctionDef {
        vis: private(),
        name: "inc_value".into(),
        throws: None,
        args: vec![arg("mut_obj", reference_mut(object("MutableClass")))],
        return_type: None,
        body: block([set(
            ident("mut_obj"),
            "value",
            add(int32(), lit::int32(1), ident("mut_obj.value")),
        )]),
    });

    module.add_test(
        "test_mutate_fields",
        [
            mut_val(
                "mut_obj",
                object("MutableClass"),
                create_class("MutableClass", []),
            ),
            assert_eq(ident("mut_obj.value"), lit::int32(0)),
            method_call(ident("mut_obj"), "inc_value", []).into_statement(),
            assert_eq(ident("mut_obj.value"), lit::int32(1)),
            call("inc_value", [ref_mut("mut_obj")]).into_statement(),
            assert_eq(ident("mut_obj.value"), lit::int32(2)),
        ],
    );

    module
}
