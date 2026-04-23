/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[uniffi::export]
pub fn roundtrip_bytes(a: Vec<u8>) -> Vec<u8> {
    a
}

#[uniffi::export]
pub fn sum_bytes_procmacro(buf: &[u8]) -> u32 {
    buf.iter().map(|b| *b as u32).sum()
}

#[uniffi::export]
pub fn first_byte_procmacro(buf: &[u8]) -> Option<u8> {
    buf.first().copied()
}
