/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Pointer FFI versions of builtin FFI functions

use std::slice;

use super::{ffi_buffer_size, FfiSerialize, RustBuffer};

/// This helper allocates a new byte buffer owned by the Rust code, and returns it
/// to the foreign-language code as a `RustBuffer` struct. Callers must eventually
/// free the resulting buffer, either by explicitly calling [`uniffi_rustbuffer_free`] defined
/// below, or by passing ownership of the buffer back into Rust code.
///
/// # Safety
///
/// The `ffi_buffer` argument must be serialized according to the Pointer FFI protocol.
pub unsafe fn rustbuffer_alloc(ffi_buffer: *mut u8) {
    let mut args_buf = slice::from_raw_parts(ffi_buffer, ffi_buffer_size!((u64)));
    let size = <u64 as FfiSerialize>::read(&mut args_buf);

    let rust_buffer = RustBuffer::new_with_size(size);
    let mut uniffi_return_buf =
        ::std::slice::from_raw_parts_mut(ffi_buffer, ffi_buffer_size!((RustBuffer)));
    RustBuffer::write(&mut uniffi_return_buf, rust_buffer);
}

/// Free a byte buffer that had previously been passed to the foreign language code.
///
/// # Safety
///
/// The `ffi_buffer` argument must be serialized according to the Pointer FFI protocol.
pub unsafe fn rustbuffer_free(ffi_buffer: *mut u8) {
    let mut args_buf = slice::from_raw_parts(ffi_buffer, ffi_buffer_size!((RustBuffer)));
    let rust_buffer = <RustBuffer as FfiSerialize>::read(&mut args_buf);
    rust_buffer.destroy();
}
