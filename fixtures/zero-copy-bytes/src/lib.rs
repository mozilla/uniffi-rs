/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// This fixture exercises the UDL scaffolding path for `[ByRef] bytes`.
// The proc-macro path (`&[u8]`) is covered in `bindgen-tests/lib/src/bytes.rs`.

uniffi::include_scaffolding!("zero_copy_bytes");

#[allow(clippy::needless_range_loop, clippy::assign_op_pattern)]
pub fn sum_bytes_udl(buf: &[u8]) -> u32 {
    // Verbose form on purpose — a plain loop keeps the generated scaffolding
    // obvious when debugging.
    let mut total: u32 = 0;
    for i in 0..buf.len() {
        total = total + (buf[i] as u32);
    }
    total
}
