/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[derive(Debug, Clone)]
struct SnakeCaseObject {
    id: u32,
}

#[derive(Debug, Clone)]
struct CamelCaseObject {
    id: u32,
}

fn camel_case_method(id: u32) -> CamelCaseObject {
    CamelCaseObject { id }
}

fn snake_case_method(id: u32) -> SnakeCaseObject {
    SnakeCaseObject { id }
}

include!(concat!(env!("OUT_DIR"), "/naming-conventions.uniffi.rs"));
