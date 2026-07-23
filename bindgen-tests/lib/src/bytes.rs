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

/// Zero-copy `&mut [u8]` — proc-macro path. Overwrites `buf` with a known
/// pattern (`buf[i] = i as u8`). The caller observes the writes in place.
#[uniffi::export]
pub fn fill_bytes_procmacro(buf: &mut [u8]) {
    for (i, b) in buf.iter_mut().enumerate() {
        *b = i as u8;
    }
}

/// Zero-copy `&mut [u8]` — proc-macro path. Increments every byte, wrapping.
#[uniffi::export]
pub fn increment_bytes_procmacro(buf: &mut [u8]) {
    for b in buf.iter_mut() {
        *b = b.wrapping_add(1);
    }
}
