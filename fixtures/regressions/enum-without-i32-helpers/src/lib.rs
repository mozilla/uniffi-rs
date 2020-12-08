/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

pub enum Which {
    Yeah,
    Nah,
}

pub fn which(arg: bool) -> Which {
    if arg {
        Which::Yeah
    } else {
        Which::Nah
    }
}

include!(concat!(env!("OUT_DIR"), "/test.uniffi.rs"));
