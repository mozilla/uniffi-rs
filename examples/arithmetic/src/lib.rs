/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use uniffi_macros::*;

#[uniffi_export_enum]
pub enum Overflow {
    WRAPPING,
    SATURATING,
}

#[uniffi_export_fn]
pub fn add(a: u64, b: u64, overflow: Overflow) -> u64 {
    match overflow {
        Overflow::WRAPPING => a.overflowing_add(b).0,
        Overflow::SATURATING => a.saturating_add(b),
    }
}

#[uniffi_export_fn]
pub fn sub(a: u64, b: u64, overflow: Overflow) -> u64 {
    match overflow {
        Overflow::WRAPPING => a.overflowing_sub(b).0,
        Overflow::SATURATING => a.saturating_sub(b),
    }
}
