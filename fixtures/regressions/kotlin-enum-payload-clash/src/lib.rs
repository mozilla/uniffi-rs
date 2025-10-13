/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Eq, uniffi::Record)]
pub struct Amount {
    pub value: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, uniffi::Enum)]
pub enum AmountOrMax {
    Amount { amount: Amount },
    Max,
}

#[derive(Clone, Debug, PartialEq, Eq, uniffi::Enum)]
pub enum ComplexValue {
    OptionalAmount {
        maybe_amount: Option<Amount>,
    },
    Amounts {
        amounts: Vec<Amount>,
    },
    AmountMap {
        amount_map: HashMap<String, Amount>,
    },
    NestedOptional {
        maybe_amounts: Option<Vec<Amount>>,
    },
    NestedMap {
        nested_map: HashMap<String, Vec<Amount>>,
    },
}

#[uniffi::export]
pub fn get_value() -> AmountOrMax {
    AmountOrMax::Amount {
        amount: Amount { value: 100 },
    }
}

#[uniffi::export]
pub fn get_complex_value() -> ComplexValue {
    ComplexValue::Amounts {
        amounts: vec![Amount { value: 1 }, Amount { value: 2 }],
    }
}

uniffi::setup_scaffolding!("regression_kotlin_enum_payload_clash");
