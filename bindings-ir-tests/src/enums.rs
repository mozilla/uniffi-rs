/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::test_module;
use bindings_ir::ir::*;

pub fn enum_tests() -> Module {
    let mut module = test_module();

    module.add_enum(EnumDef {
        vis: private(),
        name: "EnumWithoutFields".into(),
        variants: vec![
            Variant {
                name: "A".into(),
                fields: vec![],
            },
            Variant {
                name: "B".into(),
                fields: vec![],
            },
        ],
    });
    module.add_enum(EnumDef {
        vis: public(),
        name: "EnumWithFields".into(),
        variants: vec![
            Variant {
                name: "A".into(),
                fields: vec![field("x", int32()), field("y", int32())],
            },
            Variant {
                name: "B".into(),
                fields: vec![field("str", string())],
            },
        ],
    });

    module.add_test(
        "test_fieldless",
        [
            var("match_arm", string(), lit::string("none")),
            val(
                "test_enum",
                object("EnumWithoutFields"),
                create_enum("EnumWithoutFields", "A", []),
            ),
            match_enum(
                ident("test_enum"),
                "EnumWithoutFields",
                [
                    arm::variant("A", [], [assign("match_arm", lit::string("a"))]),
                    arm::variant("B", [], [assign("match_arm", lit::string("b"))]),
                ],
            ),
            assert_eq(ident("match_arm"), lit::string("a")),
        ],
    );
    module.add_test(
        "test_int_fields",
        [
            var("match_arm", string(), lit::string("none")),
            val(
                "test_enum",
                object("EnumWithFields"),
                create_enum("EnumWithFields", "A", [lit::int(1), lit::int(2)]),
            ),
            match_enum(
                ident("test_enum"),
                "EnumWithFields",
                [
                    arm::variant(
                        "A",
                        [String::from("x"), String::from("y")],
                        [
                            assign("match_arm", lit::string("a")),
                            assert_eq(ident("x"), lit::int(1)),
                            assert_eq(ident("y"), lit::int(2)),
                        ],
                    ),
                    arm::variant(
                        "B",
                        [String::from("str")],
                        [assign("match_arm", lit::string("b"))],
                    ),
                ],
            ),
            assert_eq(ident("match_arm"), lit::string("a")),
        ],
    );
    module.add_test(
        "test_string_fields",
        [
            var("match_arm", string(), lit::string("none")),
            val(
                "test_enum",
                object("EnumWithFields"),
                create_enum("EnumWithFields", "B", [lit::string("something")]),
            ),
            match_enum(
                ident("test_enum"),
                "EnumWithFields",
                [
                    arm::variant(
                        "A",
                        [String::from("x"), String::from("y")],
                        [assign("match_arm", lit::string("a"))],
                    ),
                    arm::variant(
                        "B",
                        [String::from("str")],
                        [
                            assign("match_arm", lit::string("b")),
                            assert_eq(ident("str"), lit::string("something")),
                        ],
                    ),
                ],
            ),
            assert_eq(ident("match_arm"), lit::string("b")),
        ],
    );
    module.add_test(
        "test_default",
        [
            var("match_arm", string(), lit::string("none")),
            val(
                "test_enum",
                object("EnumWithFields"),
                create_enum("EnumWithFields", "A", [lit::int(1), lit::int(2)]),
            ),
            match_enum(
                ident("test_enum"),
                "EnumWithFields",
                [
                    arm::variant("B", [String::from("x"), String::from("y")], []),
                    arm::variant_default([assign("match_arm", lit::string("default_matched"))]),
                ],
            ),
            assert_eq(ident("match_arm"), lit::string("default_matched")),
        ],
    );

    module
}
