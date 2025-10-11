/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Amount {
    pub value: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AmountOrMax {
    Amount { amount: Amount },
    Max,
}

pub fn get_value() -> AmountOrMax {
    AmountOrMax::Amount {
        amount: Amount { value: 100 },
    }
}

uniffi::include_scaffolding!("test");
