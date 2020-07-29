/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// Flag to let calling code specify a particular behaviour of integer overflow,
// since rust has such nice support for these.
pub enum Overflow {
    WRAPPING,
    SATURATING,
}

pub fn add(a: u64, b: u64, overflow: Overflow) -> u64 {
    match overflow {
        Overflow::WRAPPING => a.overflowing_add(b).0,
        Overflow::SATURATING => a.saturating_add(b),
    }
}

pub fn sub(a: u64, b: u64, overflow: Overflow) -> u64 {
    match overflow {
        Overflow::WRAPPING => a.overflowing_sub(b).0,
        Overflow::SATURATING => a.saturating_sub(b),
    }
}

pub fn equal(a: u64, b: u64) -> bool {
  a == b
}

include!(concat!(env!("OUT_DIR"), "/arithmetic.uniffi.rs"));
