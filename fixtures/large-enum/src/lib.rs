/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use uniffi::{Enum, Record};

#[derive(Clone, Debug, Enum)]
pub enum Instruction {
    CallMethod1 {
        object: Value,
        method_name: Value,
        args: Value,
    },
    CallMethod2 {
        object: Value,
        method_name: Value,
        args: Value,
    },
    CallMethod3 {
        object: Value,
        method_name: Value,
        args: Value,
    },
    CallMethod4 {
        object: Value,
        method_name: Value,
        args: Value,
    },
    CallMethod5 {
        object: Value,
        method_name: Value,
        args: Value,
    },
    CallMethod6 {
        object: Value,
        method_name: Value,
        args: Value,
    },
    CallMethod10 {
        object: Value,
        method_name: Value,
        args: Value,
    },
    CallMethod11 {
        object: Value,
        method_name: Value,
        args: Value,
    },
    CallMethod12 {
        object: Value,
        method_name: Value,
        args: Value,
    },
    CallMethod13 {
        object: Value,
        method_name: Value,
        args: Value,
    },
    CallMethod14 {
        object: Value,
        method_name: Value,
        args: Value,
    },
    CallMethod15 {
        object: Value,
        method_name: Value,
        args: Value,
    },
    CallMethod16 {
        object: Value,
        method_name: Value,
        args: Value,
    },
    CallMethod17 {
        object: Value,
        method_name: Value,
        args: Value,
    },
    CallMethod18 {
        object: Value,
        method_name: Value,
        args: Value,
    },
    CallMethod19 {
        object: Value,
        method_name: Value,
        args: Value,
    },
    CallMethod20 {
        object: Value,
        method_name: Value,
        args: Value,
    },
    CallMethod21 {
        object: Value,
        method_name: Value,
        args: Value,
    },
    CallMethod22 {
        object: Value,
        method_name: Value,
        args: Value,
    },
    CallMethod23 {
        object: Value,
        method_name: Value,
        args: Value,
    },
    CallMethod24 {
        object: Value,
        method_name: Value,
        args: Value,
    },
    CallMethod25 {
        object: Value,
        method_name: Value,
        args: Value,
    },
    CallMethod26 {
        object: Value,
        method_name: Value,
        args: Value,
    },
    CallMethod27 {
        object: Value,
        method_name: Value,
        args: Value,
    },
}

#[derive(Clone, Debug, Enum)]
pub enum Value {
    String {
        value: String,
    },
    Bool {
        value: bool,
    },

    U8 {
        value: u8,
    },
    U16 {
        value: u16,
    },
    U32 {
        value: u32,
    },
    U64 {
        value: u64,
    },

    I8 {
        value: i8,
    },
    I16 {
        value: i16,
    },
    I32 {
        value: i32,
    },
    I64 {
        value: i64,
    },

    Enum {
        discriminator: u8,
        fields: Vec<Value>,
    },
    HeterogenousCollection {
        elements: Vec<Value>,
    },
    HomogeneousCollection {
        elements: Vec<Value>,
    },
    Map {
        entries: Vec<MapEntry>,
    },

    PublicKey {
        value: Vec<u8>,
    },

    Signature {
        value: Vec<u8>,
    },
}

#[derive(Clone, Debug, Record)]
pub struct MapEntry {
    pub key: Value,
    pub value: Value,
}

uniffi::include_scaffolding!("large_enum");
