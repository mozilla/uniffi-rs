/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! End-to-end UDL fixture for zero-copy `&mut [u8]` arguments declared with
//! `[ByMutRef] bytes` in the UDL. The scaffolding generator lowers such an
//! argument to `&mut [u8]`, which flows through the proc-macro machinery so
//! that Rust writes land in the caller's buffer in place.

/// Overwrites `buf` with a known pattern (`buf[i] = i as u8`). The caller
/// observes the writes in place.
pub fn fill_bytes_udl(buf: &mut [u8]) {
    for (i, b) in buf.iter_mut().enumerate() {
        *b = i as u8;
    }
}

/// Increments every byte, wrapping. The caller observes the writes in place.
pub fn increment_bytes_udl(buf: &mut [u8]) {
    for b in buf.iter_mut() {
        *b = b.wrapping_add(1);
    }
}

uniffi::include_scaffolding!("mut-bytes");
