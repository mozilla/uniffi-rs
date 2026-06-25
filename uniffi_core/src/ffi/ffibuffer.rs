/* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Manage buffers that contain FFI values
//!
//! These are used to pass dynamically sized values like vecs
//! and hash maps across the FFI.
//! `ffibuffer` is a newer module, currently only used by `uniffi-bindgen-kotlin-jni.
//! It replaces the older `RustBuffer` code.
//!
//! The functions in this module input raw `*mut u8` pointers.
//! It doesn't have any concept of the "current buffer position".
//! Instead, the generated code does its own pointer arithmetic.
//!
//! All functions return `Result<>` values even though none of them actually return an `Err` value.
//! This is because read/write functions for user types may return errors.
//! For example, the custom type read function may fail when lifting the lowered type fails.
//!
//! FFI buffers have an unsafe API, the generated code is responsible for ensuring:
//! * Reads/writes never happen beyond the allocated capacity
//! * Reads/writes are properly aligned
//! * Reads only happen when the other side of the FFI has written a value to the current address.

use std::{alloc, mem, ptr};

use anyhow::bail;

use crate::Result;

/// Allocate a buffer pointer
pub fn alloc(capacity: usize) -> Result<*mut u8> {
    if capacity == 0 {
        // zero-capacity allocations are UB, return a null pointer instead
        return Ok(ptr::null_mut());
    }
    if capacity > isize::MAX as usize {
        bail!("isize overfow when allocating a FFI buffer (attempted capacity: {capacity})");
    }
    // Use alignment=8 when allocating buffers, that's enough to ensure all items are aligned.
    //
    // Safety:
    //
    // * capacity is non-zero
    // * capacity does not overflow `isize`.
    // * alignment is a non-zero multiple of 8
    let ptr = unsafe { alloc::alloc(alloc::Layout::from_size_align_unchecked(capacity, 8)) };
    trace!("ffibuffer::alloc {ptr:?} (capacity: {capacity})");
    Ok(ptr)
}

/// Free a buffer allocated with [alloc]
///
/// # Safety
///
/// * ptr must have come from a [alloc] call with the same capacity
/// * ptr must not be used again after this call
pub unsafe fn free(ptr: *mut u8, capacity: usize) {
    if capacity == 0 {
        // [alloc] doesn't actually allocate anything in this case
        return;
    }

    trace!("ffibuffer::free {ptr:?}");
    // Safety:
    //
    // * Layout::from_size_align_unchecked is safe for the same reasons as in the constructor.
    // * The data and layout values are the same as the ones passed in to [alloc]
    unsafe {
        alloc::dealloc(ptr, alloc::Layout::from_size_align_unchecked(capacity, 8));
    }
}

macro_rules! define_simple_read_and_write_fns {
    ($ty:ty, $read_fn:ident, $write_fn:ident) => {
        /// Read a value to the buffer
        ///
        /// # Safety
        ///
        /// * There must be at enough capacity remaining in the pointer to read the type
        /// * The pointer must be properly aligned
        /// * The other side of the FFI must have written a value to the current address
        pub unsafe fn $read_fn(ptr: *mut u8) -> Result<$ty> {
            unsafe { Ok(ptr.cast::<$ty>().read()) }
        }

        /// Write a value to the buffer
        ///
        /// # Safety
        ///
        /// * There must be at enough capacity remaining in the pointer to write the type
        /// * The pointer must be properly aligned
        pub unsafe fn $write_fn(ptr: *mut u8, value: $ty) -> Result<()> {
            unsafe {
                ptr.cast::<$ty>().write(value);
                Ok(())
            }
        }
    };
}

define_simple_read_and_write_fns!(u8, read_u8, write_u8);
define_simple_read_and_write_fns!(i8, read_i8, write_i8);
define_simple_read_and_write_fns!(u16, read_u16, write_u16);
define_simple_read_and_write_fns!(i16, read_i16, write_i16);
define_simple_read_and_write_fns!(u32, read_u32, write_u32);
define_simple_read_and_write_fns!(i32, read_i32, write_i32);
define_simple_read_and_write_fns!(u64, read_u64, write_u64);
define_simple_read_and_write_fns!(i64, read_i64, write_i64);
define_simple_read_and_write_fns!(f32, read_f32, write_f32);
define_simple_read_and_write_fns!(f64, read_f64, write_f64);

/// Read a bool value
///
/// # Safety
///
/// * There must be at least 1 byte of capacity remaining in the pointer.
/// * The other side of the FFI must have written a bool to the current address
pub unsafe fn read_bool(ptr: *mut u8) -> Result<bool> {
    Ok(unsafe { read_u8(ptr)? == 1 })
}

/// Write a bool value
///
/// # Safety
///
/// * There must be at least 1 byte of capacity remaining in the pointer.
pub unsafe fn write_bool(ptr: *mut u8, val: bool) -> Result<()> {
    unsafe { write_u8(ptr, if val { 1 } else { 0 }) }
}

/// Read a string value
///
/// Strings are deconstructed into their raw parts, the raw parts are casted to `u64` values,
/// then written to the buffer.
/// Strings contain a data, length, and capacity field
/// so the total size is 24 bytes and the alignment is 8.
///
/// # Safety
///
/// * There must be at least 24 bytes of capacity remaining in the pointer.
/// * The position must be aligned to 8 bytes
/// * The other side of the FFI must have written a string to the current address
pub unsafe fn read_string(ptr: *mut u8) -> Result<String> {
    // Safety:
    //
    // * ptr is properly aligned and has enough space left
    // * We assume the foreign side wrote the correct value to the buffer
    Ok(unsafe {
        let ptr = ptr.cast::<u64>();
        let data: u64 = ptr.read();
        let length: u64 = ptr.add(1).read();
        let capacity: u64 = ptr.add(2).read();
        trace!("ffibuffer::read_string ({ptr:?} -> {data:x}, {length}, {capacity})");
        String::from_raw_parts(
            ptr::with_exposed_provenance_mut(data as usize),
            length as usize,
            capacity as usize,
        )
    })
}

/// Write a string value
///
/// Strings are deconstructed into their raw parts, the raw parts are casted to `u64` values,
/// then written to the buffer.
/// Strings contain a data, length, and capacity field
/// so the total size is 24 bytes and the alignment is 8.
///
/// # Safety
///
/// * There must be at least 24 bytes of capacity remaining in the pointer.
/// * The position must be aligned to 8 bytes
pub unsafe fn write_string(ptr: *mut u8, mut value: String) -> Result<()> {
    // Safety:
    //
    // * ptr is properly aligned and has enough space left
    unsafe {
        let data = value.as_mut_ptr().expose_provenance() as u64;
        let length = value.len() as u64;
        let capacity = value.capacity() as u64;
        trace!("ffibuffer::write_string  ({ptr:?} -> {data:x}, {length}, {capacity})");
        // Leak the string, then write the components to the buffer.
        mem::forget(value);
        let ptr = ptr.cast::<u64>();
        ptr.write(data);
        ptr.add(1).write(length);
        ptr.add(2).write(capacity);
        Ok(())
    }
}

/// Read a nested FFI buffer
///
/// Like strings, these are deconstructed into their raw parts, casted to `u64` values,
/// then written to the buffer.
/// Buffer only contain a data and size field so
/// the total size is 16 bytes and the alignment is 8.
///
/// # Safety
///
/// * There must be at least 16 bytes of capacity remaining in the pointer.
/// * The position must be aligned to 8 bytes
/// * The other side of the FFI must have written a buffer to the current address
pub unsafe fn read_buffer(ptr: *mut u8) -> Result<(*mut u8, usize)> {
    // Safety:
    //
    // * ptr is properly aligned and has enough space left
    // * We assume the foreign side wrote the correct value to the buffer
    Ok(unsafe {
        let ptr = ptr.cast::<u64>();
        let data: u64 = ptr.read();
        let size: u64 = ptr.add(1).read();
        trace!("ffibuffer::read_buffer ({ptr:?} -> {data:x}, {size})");
        (
            ptr::with_exposed_provenance_mut(data as usize),
            size as usize,
        )
    })
}

/// Write a buffer value
///
/// Like strings, these are deconstructed into their raw parts, casted to `u64` values,
/// then written to the buffer.
/// Buffer only contain a data and size field so
/// the total size is 16 bytes and the alignment is 8.
///
/// # Safety
///
/// * There must be at least 16 bytes of capacity remaining in the pointer.
/// * The position must be aligned to 8 bytes
pub unsafe fn write_buffer(ptr: *mut u8, data: *mut u8, size: usize) -> Result<()> {
    // Safety:
    //
    // * ptr is properly aligned and has enough space left
    unsafe {
        let data = data.expose_provenance() as u64;
        let size = size as u64;
        trace!("ffibuffer::write_buffer ({ptr:?} -> {data:x}, {size})");
        let ptr = ptr.cast::<u64>();
        ptr.write(data);
        ptr.add(1).write(size);
        Ok(())
    }
}
