/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Methods with complex signatures on records, enums and objects.
//!
//! The `records`, `enums` and `interfaces` fixtures test the simple `roundtrip()` style methods.
//! This one tests that the different kinds of arguments work on each kind of receiver.

use std::sync::Arc;

use crate::enums::{EnumNoData, EnumWithData};
use crate::interfaces::TestInterface;
use crate::records::SimpleRec;

#[derive(uniffi::Record)]
pub struct MethodTestRec {
    pub value: u8,
}

#[derive(uniffi::Enum)]
pub enum MethodTestEnumNoData {
    A,
    B,
}

#[derive(uniffi::Enum)]
pub enum MethodTestEnumWithData {
    A { value: u8 },
    B(String),
    C,
}

#[derive(uniffi::Object)]
pub struct MethodTestInterface {
    value: u8,
}

#[uniffi::export]
impl MethodTestRec {
    #[allow(clippy::too_many_arguments)]
    pub fn describe_call(
        &self,
        number: u8,
        number_ref: &u8,
        string: String,
        string_ref: &str,
        rec: SimpleRec,
        rec_ref: &SimpleRec,
        enum_no_data: EnumNoData,
        enum_with_data: EnumWithData,
        enum_with_data_ref: &EnumWithData,
        interface: Arc<TestInterface>,
        interface_ref: &TestInterface,
    ) -> String {
        describe_call(
            format!("rec({})", self.value),
            number,
            number_ref,
            &string,
            string_ref,
            &rec,
            rec_ref,
            &enum_no_data,
            &enum_with_data,
            enum_with_data_ref,
            &interface,
            interface_ref,
        )
    }
}

#[uniffi::export]
impl MethodTestEnumNoData {
    #[allow(clippy::too_many_arguments)]
    pub fn describe_call(
        &self,
        number: u8,
        number_ref: &u8,
        string: String,
        string_ref: &str,
        rec: SimpleRec,
        rec_ref: &SimpleRec,
        enum_no_data: EnumNoData,
        enum_with_data: EnumWithData,
        enum_with_data_ref: &EnumWithData,
        interface: Arc<TestInterface>,
        interface_ref: &TestInterface,
    ) -> String {
        let receiver = match self {
            Self::A => "enum_no_data(A)",
            Self::B => "enum_no_data(B)",
        };
        describe_call(
            receiver.to_string(),
            number,
            number_ref,
            &string,
            string_ref,
            &rec,
            rec_ref,
            &enum_no_data,
            &enum_with_data,
            enum_with_data_ref,
            &interface,
            interface_ref,
        )
    }
}

#[uniffi::export]
impl MethodTestEnumWithData {
    #[allow(clippy::too_many_arguments)]
    pub fn describe_call(
        &self,
        number: u8,
        number_ref: &u8,
        string: String,
        string_ref: &str,
        rec: SimpleRec,
        rec_ref: &SimpleRec,
        enum_no_data: EnumNoData,
        enum_with_data: EnumWithData,
        enum_with_data_ref: &EnumWithData,
        interface: Arc<TestInterface>,
        interface_ref: &TestInterface,
    ) -> String {
        let receiver = match self {
            Self::A { value } => format!("enum_with_data(A({value}))"),
            Self::B(value) => format!("enum_with_data(B({value}))"),
            Self::C => "enum_with_data(C)".to_string(),
        };
        describe_call(
            receiver,
            number,
            number_ref,
            &string,
            string_ref,
            &rec,
            rec_ref,
            &enum_no_data,
            &enum_with_data,
            enum_with_data_ref,
            &interface,
            interface_ref,
        )
    }
}

#[uniffi::export]
impl MethodTestInterface {
    #[uniffi::constructor]
    pub fn new(value: u8) -> Self {
        Self { value }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn describe_call(
        &self,
        number: u8,
        number_ref: &u8,
        string: String,
        string_ref: &str,
        rec: SimpleRec,
        rec_ref: &SimpleRec,
        enum_no_data: EnumNoData,
        enum_with_data: EnumWithData,
        enum_with_data_ref: &EnumWithData,
        interface: Arc<TestInterface>,
        interface_ref: &TestInterface,
    ) -> String {
        describe_call(
            format!("interface({})", self.value),
            number,
            number_ref,
            &string,
            string_ref,
            &rec,
            rec_ref,
            &enum_no_data,
            &enum_with_data,
            enum_with_data_ref,
            &interface,
            interface_ref,
        )
    }
}

#[allow(clippy::too_many_arguments)]
fn describe_call(
    receiver: String,
    number: u8,
    number_ref: &u8,
    string: &str,
    string_ref: &str,
    rec: &SimpleRec,
    rec_ref: &SimpleRec,
    enum_no_data: &EnumNoData,
    enum_with_data: &EnumWithData,
    enum_with_data_ref: &EnumWithData,
    interface: &TestInterface,
    interface_ref: &TestInterface,
) -> String {
    [
        receiver,
        number.to_string(),
        number_ref.to_string(),
        string.to_string(),
        string_ref.to_string(),
        format!("rec({})", rec.a),
        format!("rec({})", rec_ref.a),
        describe_enum_no_data(enum_no_data).to_string(),
        describe_enum_with_data(enum_with_data),
        describe_enum_with_data(enum_with_data_ref),
        format!("interface({})", interface.get_value()),
        format!("interface({})", interface_ref.get_value()),
    ]
    .join(" ")
}

fn describe_enum_no_data(en: &EnumNoData) -> &'static str {
    match en {
        EnumNoData::A => "A",
        EnumNoData::B => "B",
        EnumNoData::C => "C",
    }
}

fn describe_enum_with_data(en: &EnumWithData) -> String {
    match en {
        EnumWithData::A { value, value2 } => format!("A({value},{value2})"),
        EnumWithData::B(string, number) => format!("B({string},{number})"),
        EnumWithData::C => "C".to_string(),
    }
}
