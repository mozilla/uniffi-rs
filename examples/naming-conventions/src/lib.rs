/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[derive(Debug, Clone)]
struct snake_case_object {
    id: u32,
}

#[derive(Debug, Clone)]
struct CamelCaseObject {
    id: u32,
}

enum Case {
    snake_case,
    UpperCamelCase,
    camelCase,
    SHOUTY_SNAKE_CASE,
}

fn camelCaseMethod(id: u32, _from: Case) -> CamelCaseObject {
    CamelCaseObject { id }
}

fn snake_case_method(id: u32, _from: Case) -> snake_case_object {
    snake_case_object { id }
}

fn get_snake_case() -> Case {
    Case::snake_case
}

fn getCamelCase() -> Case {
    Case::UpperCamelCase
}

include!(concat!(env!("OUT_DIR"), "/naming-conventions.uniffi.rs"));
