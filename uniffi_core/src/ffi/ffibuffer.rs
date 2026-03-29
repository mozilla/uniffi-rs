/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! API for reading and writing to FFI buffers
//!
//! FFI buffers are a reasonably efficient general purpose mechanism for passing FFI values.
//! New bindings generators can use FFI buffers as a starting point for their FFI.
//!
//! FFI buffers are usually not the absolute fastest way to pass values
//! and bindings generators may look to replace them in some cases.
//! For example, it's probably faster to pass primitive values as direct arguments/return values.
//! Bindings generators are encouraged to special case certain types of values for performance
//! and fall back to FFI buffers for the other cases.
//!
//! The data layout for a FFI buffer is a linked list of smaller buffers, called "mini buffers".
//! Each mini buffer store a pointer to the next buffer in the last 8 bytes.
//! Also, each mini buffer is twice the size of the previous one to minimize allocations.
//! All items in the buffer are stored in native endian and aligned.
//!
//! This has some nice benefits:
//! * Buffers are dynamically sized without requiring a copy/reallocation.
//! * We can represent the entire buffer using a single pointer
//!   which is easy to pass across the FFI as an argument/return value.
//! * It's possible to avoid allocations in many cases by keeping
//!   the "head" of the list around in a thread-local variable.
//!   This means we only need to allocate when the call requires
//!   more than `BASE_MINI_BUFFER_SIZE` bytes.
//!   (Not implemented yet, but I plan to do it and expect it will improve performance).
//!
//! The main downside is that random access is complicated / potentially slow,
//! but that's not needed when reading FFI values.
//!
//! This module provides 2 APIs:
//! * The high-level [FfiBufferCursor] API.
//!   This API is used by Rust code and can be used by foreign bindings
//!   if it's easy to store a struct instance in a foreign value.
//!   For example, a Python extension type could store it fairly easy.
//! * A low-level API built around top-level functions that input pointers and return
//!   (`mini_buffer_next`, `read_string_from_pointer`, etc).
//!   This API is for foreign bindings that can't effectively store a Rust struct.
//!   For example, JNI requires all data to be stored in the JVM object.
//!   This means if we want to use a [FfiBufferCursor],
//!   we'd have to marshal all 3 fields of it's across the FFI for each read/write.
//!   In that case, it's better to manage the data in a Java/Kotlin class
//!   and reimplements some of the [FfiBufferCursor] logic.

use std::{
    alloc::{alloc, dealloc, Layout},
    mem, ptr,
};

use crate::Result;

/// Size of the initial buffer in FFI buffer list (in bytes).
const BASE_MINI_BUFFER_SIZE: usize = 256;

/// FFI buffer wrapper
pub struct FfiBuffer {
    head: *mut u8,
}

impl Default for FfiBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl FfiBuffer {
    pub fn new() -> Self {
        Self {
            head: ffi_buffer_alloc(),
        }
    }

    pub fn with_cursor<F, T>(&mut self, f: F) -> Result<T>
    where
        F: FnOnce(&mut FfiBufferCursor) -> Result<T>,
    {
        // Safety:
        // * self.head points to a valid FFI buffer
        // * This buffer won't be used while the cursor is alive since we take a mutable reference.
        let mut cursor = unsafe { FfiBufferCursor::new(self.head) };
        f(&mut cursor)
    }

    /// Create a pointer to pass across the FFI
    ///
    /// # Safety
    ///
    /// The buffer must only be used by one side of the FFI at once.
    pub unsafe fn as_ptr(&self) -> *mut u8 {
        self.head
    }

    pub fn into_ptr(self) -> *mut u8 {
        self.head
    }

    pub fn from_ptr(head: *mut u8) -> Self {
        Self { head }
    }

    pub fn free(self) {
        ffi_buffer_free(self.head)
    }
}

/// Allocate a new FFI buffer
///
/// This is the low-level API for managing FFI buffers.
/// It returns the head pointer, which points to the start of the first mini buffer.
///
/// When using the low-level API, foreign bindings should track:
/// * This pointer, so that it can eventually be passed to `ffi_buffer_free`
/// * A pointer at the position in the current mini buffer
/// * The size of the current mini buffer:
///    * This mini buffer is 256 bytes
///    * Each mini buffer returned from `mini_buffer_next` is twice the size of the last one.
/// * A pointer at the end position in the current mini buffer (8 bytes from the end of the buffer).
///
/// See the [FfiBufferCursor] code for how this should be done.
#[allow(clippy::let_and_return)]
pub fn ffi_buffer_alloc() -> *mut u8 {
    let head = mini_buffer_alloc(BASE_MINI_BUFFER_SIZE);
    trace!("ffi_buffer_alloc: {head:?}");
    head
}

/// Free a FFI buffer
pub fn ffi_buffer_free(head: *mut u8) {
    trace!("ffi_buffer_free: {head:?}");
    mini_buffer_free(head, BASE_MINI_BUFFER_SIZE)
}

/// Move to the next mini buffer, allocating a new one if needed.
///
/// Returns a pointer to the next minibuffer.
/// The size of the mini buffer will be 2*size
///
/// # Safety
///
/// `end_ptr` must be the end pointer from a valid minibuffer
pub unsafe fn mini_buffer_next(end_ptr: *mut u8, size: usize) -> *mut u8 {
    // Safety:
    // `end_ptr` is valid for pointer reads
    let mut next_ptr = unsafe { end_ptr.cast::<*mut u8>().read() };
    if next_ptr.is_null() {
        next_ptr = mini_buffer_alloc(size * 2);
        // Safety:
        // `end_ptr` is valid for pointer writes
        unsafe {
            end_ptr.cast::<*mut u8>().write(next_ptr);
        }
    }
    trace!("mini_buffer_next: {next_ptr:?}");
    next_ptr
}

/// Read a string value from buffer pointer
///
/// This is a lower-level version of `FfiBufferCursor::read_string`.
/// This function is exposed for foreign bindings that manage their own FFI buffers.
///
/// # Safety
///
/// The pointer must be located at a previous `write_string_to_pointer` or
/// `FfiBufferCursor::write_string` call.
pub unsafe fn read_string_from_pointer(ptr: *mut u8) -> Result<String> {
    // Safety:
    // The caller ensures that `ptr` points to the correct location
    unsafe {
        let ptr = ptr.cast::<u64>();
        let data: u64 = ptr.read();
        let length: u64 = ptr.add(1).read();
        let capacity: u64 = ptr.add(2).read();
        trace!("read_string_from_pointer ({ptr:?} {data:x}, {length}, {capacity})");
        Ok(String::from_raw_parts(
            ptr::with_exposed_provenance_mut(data as usize),
            length as usize,
            capacity as usize,
        ))
    }
}

/// Write a string value to buffer pointer
///
/// This is a lower-level version of `FfiBufferCursor::write_string`.
/// This function is exposed for foreign bindings that manage their own FFI buffers.
///
/// # Safety
///
/// The pointer must be aligned to 8-byte location
/// and be pointed to an allocation with at least 24 bytes of space left.
pub unsafe fn write_string_to_pointer(ptr: *mut u8, mut value: String) -> Result<()> {
    // Safety:
    // The caller ensures that `ptr` points to the correct location
    unsafe {
        let data = value.as_mut_ptr().expose_provenance() as u64;
        let length = value.len() as u64;
        let capacity = value.capacity() as u64;
        trace!("write_string_to_pointer  ({ptr:?} {data:x}, {length}, {capacity})");
        // Leak the string, then write the components to the buffer.
        //
        // This is preferable to writing the string contents for a couple reasons:
        // * This avoids at least one copy of the data
        // * It has a fixed size of 24 bytes, so we can ensure it doesn't overflow the mini buffer.
        mem::forget(value);
        let ptr = ptr.cast::<u64>();
        ptr.write(data);
        ptr.add(1).write(length);
        ptr.add(2).write(capacity);
    };
    Ok(())
}

/// Allocate a new minibuffer
fn mini_buffer_alloc(size: usize) -> *mut u8 {
    // Safety:
    //
    // The alignment is a non-zero multiple of 8
    let ptr = unsafe { alloc(Layout::from_size_align_unchecked(size, 8)) };
    // Safety:
    //
    // * The offset fits in the allocation
    // * The offset is aligned at 8-bytes, which is valid for either 32-bit or 64-bit pointers.
    unsafe {
        ptr.add(size - 8).cast::<*mut u8>().write(ptr::null_mut());
    }
    trace!("mini_buffer_alloc: {ptr:?} ({size})");
    ptr
}

fn mini_buffer_free(ptr: *mut u8, size: usize) {
    trace!("mini_buffer_free:  {ptr:?} ({size})");
    // Safety:
    //
    // * The offset fits in the allocation
    // * The offset is aligned at 8-bytes, which is valid for either 32-bit or 64-bit pointers.
    let next_ptr = unsafe { ptr.add(size - 8).cast::<*mut u8>().read() };
    if !next_ptr.is_null() {
        mini_buffer_free(next_ptr, size * 2);
    }
    // Safety:
    //
    // * The alignment is a non-zero multiple of 8
    // * The data and layout values are the same as the ones passed in to [alloc]
    unsafe { dealloc(ptr, Layout::from_size_align_unchecked(size, 8)) };
}

/// Read/Write data from a FFI buffer
///
/// This manages our position in the FFI buffer and ensures that reads/writes have proper alignment
/// and enough free space.
pub struct FfiBufferCursor {
    /// Points to the current position in the current mini buffer
    ptr: *mut u8,
    /// End position of the mini buffer, this is where the pointer to the next mini-buffer is
    /// stored.
    end: *mut u8,
    /// Size of the current mini buffer
    mini_buf_size: usize,
}

impl FfiBufferCursor {
    /// Creator a `FfiBufferCursor` for a FFI buffer pointer
    ///
    /// This effectively borrows the pointer.
    ///
    /// # Safety
    ///
    /// * `ptr` must come from a `ffi_buffer_alloc()` call
    /// * `ptr` must have been passed to `ffi_buffer_free()`
    /// * `ptr' must not be used while this `FfiBufferCursor` is alive
    pub unsafe fn new(ptr: *mut u8) -> Self {
        trace!("FfiBufferCursor::borrow_ptr {ptr:?}");
        // Safety:
        //
        // `ptr` + `BASE_SIZE` - 8 is part of the allocation for `FfiBuffer` (the very end of it).
        let end = unsafe { ptr.add(BASE_MINI_BUFFER_SIZE - 8) };
        Self {
            ptr,
            end,
            mini_buf_size: BASE_MINI_BUFFER_SIZE,
        }
    }

    // Calculate how much space is left in our current minibuf
    fn mini_buf_remaining(&self) -> usize {
        // Safety:
        //
        // `self.end` and `self.ptr` were derived from the same allocation.
        // `self.end >= self.ptr`
        unsafe { self.end.offset_from_unsigned(self.ptr) }
    }

    /// Prepare to read/write a `T` value
    ///
    /// This ensures the buffer pointer is aligned and has enough space remaining
    fn prepare(&mut self, align: usize, size: usize) {
        // Determine how much we need to add to `self.ptr` to give it the proper alignment.
        //
        // Rust provides a nice function for this.
        //
        // If a foreign language provides a modulo function that is always positive (i.e. the
        // mathematical modulo), then this can be achieved with: `offset = -self.ptr.mod(align)`
        //
        // If the foreign langugage's modulo can return negative values, like C,
        // then you need some extra code: `offset = (ptr.mod(align) + align).mod(align)`
        let mut offset = self.ptr.align_offset(align);
        if offset + size > self.mini_buf_remaining() {
            // Safety:
            // self.end is pointing to the correct position in our mini buffer
            self.ptr = unsafe { mini_buffer_next(self.end, self.mini_buf_size) };
            self.mini_buf_size *= 2;
            // Safety:
            // The offset is located in the mini buffer allocation
            self.end = unsafe { self.ptr.add(self.mini_buf_size - 8) };
            offset = 0;
        }
        // Safety:
        //
        // * We've ensured that `ptr + offset` is inside a mini buffer allocation
        unsafe {
            self.ptr = self.ptr.add(offset);
        }
    }

    /// Read a `T` value
    ///
    /// # Safety
    ///
    /// * `self.ptr` must be properly aligned and have enough space remaining.
    /// * A valid `T` value must have been written at this address by the other side of the FFI.
    unsafe fn read_primitive_unchecked<T>(&mut self) -> T {
        // Safety:
        //
        // This is safe if the caller follows our safety requirements
        unsafe {
            let val = self.ptr.cast::<T>().read();
            self.ptr = self.ptr.add(size_of::<T>());
            val
        }
    }

    /// Write a `T` value.
    ///
    /// # Safety
    ///
    /// `self.ptr` must be properly aligned and have enough space remaining.
    unsafe fn write_primitive_unchecked<T>(&mut self, value: T) {
        // Safety:
        //
        // This is safe if the caller follows our safety requirements
        unsafe {
            self.ptr.cast::<T>().write(value);
            self.ptr = self.ptr.add(size_of::<T>());
        }
    }

    /// Read a u8 value from the buffer
    pub fn read_u8(&mut self) -> Result<u8> {
        self.prepare(align_of::<u8>(), size_of::<u8>());
        // Safety:
        // * `self.prepare` ensures the pointer is properly aligned and has enough space
        // * We assume the foreign side has written a valid value at our location
        Ok(unsafe { self.read_primitive_unchecked() })
    }

    /// Read a i8 value from the buffer
    pub fn read_i8(&mut self) -> Result<i8> {
        self.prepare(align_of::<i8>(), size_of::<i8>());
        // Safety:
        // * `self.prepare` ensures the pointer is properly aligned and has enough space
        // * We assume the foreign side has written a valid value at our location
        Ok(unsafe { self.read_primitive_unchecked() })
    }

    /// Read a u16 value from the buffer
    pub fn read_u16(&mut self) -> Result<u16> {
        self.prepare(align_of::<u16>(), size_of::<u16>());
        // Safety:
        // * `self.prepare` ensures the pointer is properly aligned and has enough space
        // * We assume the foreign side has written a valid value at our location
        Ok(unsafe { self.read_primitive_unchecked() })
    }

    /// Read a i16 value from the buffer
    pub fn read_i16(&mut self) -> Result<i16> {
        self.prepare(align_of::<i16>(), size_of::<i16>());
        // Safety:
        // * `self.prepare` ensures the pointer is properly aligned and has enough space
        // * We assume the foreign side has written a valid value at our location
        Ok(unsafe { self.read_primitive_unchecked() })
    }

    /// Read a u32 value from the buffer
    pub fn read_u32(&mut self) -> Result<u32> {
        self.prepare(align_of::<u32>(), size_of::<u32>());
        // Safety:
        // * `self.prepare` ensures the pointer is properly aligned and has enough space
        // * We assume the foreign side has written a valid value at our location
        Ok(unsafe { self.read_primitive_unchecked() })
    }

    /// Read a i32 value from the buffer
    pub fn read_i32(&mut self) -> Result<i32> {
        self.prepare(align_of::<i32>(), size_of::<i32>());
        // Safety:
        // * `self.prepare` ensures the pointer is properly aligned and has enough space
        // * We assume the foreign side has written a valid value at our location
        Ok(unsafe { self.read_primitive_unchecked() })
    }

    /// Read a u64 value from the buffer
    pub fn read_u64(&mut self) -> Result<u64> {
        self.prepare(align_of::<u64>(), size_of::<u64>());
        // Safety:
        // * `self.prepare` ensures the pointer is properly aligned and has enough space
        // * We assume the foreign side has written a valid value at our location
        Ok(unsafe { self.read_primitive_unchecked() })
    }

    /// Read a i64 value from the buffer
    pub fn read_i64(&mut self) -> Result<i64> {
        self.prepare(align_of::<i64>(), size_of::<i64>());
        // Safety:
        // * `self.prepare` ensures the pointer is properly aligned and has enough space
        // * We assume the foreign side has written a valid value at our location
        Ok(unsafe { self.read_primitive_unchecked() })
    }

    /// Read a f32 value from the buffer
    pub fn read_f32(&mut self) -> Result<f32> {
        self.prepare(align_of::<f32>(), size_of::<f32>());
        // Safety:
        // * `self.prepare` ensures the pointer is properly aligned and has enough space
        // * We assume the foreign side has written a valid value at our location
        Ok(unsafe { self.read_primitive_unchecked() })
    }

    /// Read a f64 value from the buffer
    pub fn read_f64(&mut self) -> Result<f64> {
        self.prepare(align_of::<f64>(), size_of::<f64>());
        // Safety:
        // * `self.prepare` ensures the pointer is properly aligned and has enough space
        // * We assume the foreign side has written a valid value at our location
        Ok(unsafe { self.read_primitive_unchecked() })
    }

    /// Read a bool value from the buffer
    pub fn read_bool(&mut self) -> Result<bool> {
        Ok(self.read_u8()? == 1)
    }

    /// Read a bool value from the buffer, without checking for alignment / free space
    pub fn read_ptr<T>(&mut self) -> Result<*mut T> {
        Ok(ptr::with_exposed_provenance_mut(self.read_u64()? as usize))
    }

    /// Read a string value
    pub fn read_string(&mut self) -> Result<String> {
        self.prepare(8, 24);
        // Safety:
        //
        // * self.ptr is properly aligned and has enough space left
        // * We assume the foreign side wrote the correct value to the buffer
        unsafe {
            let val = read_string_from_pointer(self.ptr);
            self.ptr = self.ptr.add(24);
            val
        }
    }

    /// Write a u8 value to the buffer
    pub fn write_u8(&mut self, value: u8) -> Result<()> {
        self.prepare(align_of::<u8>(), size_of::<u8>());
        // Safety:
        // self.ptr is properly aligned and has enough space left
        unsafe { self.write_primitive_unchecked(value) };
        Ok(())
    }

    /// Write a i8 value to the buffer
    pub fn write_i8(&mut self, value: i8) -> Result<()> {
        self.prepare(align_of::<i8>(), size_of::<i8>());
        // Safety:
        // self.ptr is properly aligned and has enough space left
        unsafe { self.write_primitive_unchecked(value) };
        Ok(())
    }

    /// Write a u16 value to the buffer
    pub fn write_u16(&mut self, value: u16) -> Result<()> {
        self.prepare(align_of::<u16>(), size_of::<u16>());
        // Safety:
        // self.ptr is properly aligned and has enough space left
        unsafe { self.write_primitive_unchecked(value) };
        Ok(())
    }

    /// Write a i16 value to the buffer
    pub fn write_i16(&mut self, value: i16) -> Result<()> {
        self.prepare(align_of::<i16>(), size_of::<i16>());
        // Safety:
        // self.ptr is properly aligned and has enough space left
        unsafe { self.write_primitive_unchecked(value) };
        Ok(())
    }

    /// Write a u32 value to the buffer
    pub fn write_u32(&mut self, value: u32) -> Result<()> {
        self.prepare(align_of::<u32>(), size_of::<u32>());
        // Safety:
        // self.ptr is properly aligned and has enough space left
        unsafe { self.write_primitive_unchecked(value) };
        Ok(())
    }

    /// Write a i32 value to the buffer
    pub fn write_i32(&mut self, value: i32) -> Result<()> {
        self.prepare(align_of::<i32>(), size_of::<i32>());
        // Safety:
        // self.ptr is properly aligned and has enough space left
        unsafe { self.write_primitive_unchecked(value) };
        Ok(())
    }

    /// Write a u64 value to the buffer
    pub fn write_u64(&mut self, value: u64) -> Result<()> {
        self.prepare(align_of::<u64>(), size_of::<u64>());
        // Safety:
        // self.ptr is properly aligned and has enough space left
        unsafe { self.write_primitive_unchecked(value) };
        Ok(())
    }

    /// Write a i64 value to the buffer
    pub fn write_i64(&mut self, value: i64) -> Result<()> {
        self.prepare(align_of::<i64>(), size_of::<i64>());
        // Safety:
        // self.ptr is properly aligned and has enough space left
        unsafe { self.write_primitive_unchecked(value) };
        Ok(())
    }

    /// Write a f32 value to the buffer
    pub fn write_f32(&mut self, value: f32) -> Result<()> {
        self.prepare(align_of::<f32>(), size_of::<f32>());
        // Safety:
        // self.ptr is properly aligned and has enough space left
        unsafe { self.write_primitive_unchecked(value) };
        Ok(())
    }

    /// Write a f64 value to the buffer
    pub fn write_f64(&mut self, value: f64) -> Result<()> {
        self.prepare(align_of::<f64>(), size_of::<f64>());
        // Safety:
        // self.ptr is properly aligned and has enough space left
        unsafe { self.write_primitive_unchecked(value) };
        Ok(())
    }

    /// Write a bool value to the buffer
    pub fn write_bool(&mut self, value: bool) -> Result<()> {
        self.write_u8(value as u8)
    }

    /// Write a pointer value to the buffer
    pub fn write_ptr<T>(&mut self, value: *mut T) -> Result<()> {
        self.write_u64(value.expose_provenance() as u64)
    }

    /// Write a string to the buffer
    pub fn write_string(&mut self, value: String) -> Result<()> {
        self.prepare(8, 24);
        // Safety:
        // self.ptr is properly aligned and has enough space left
        unsafe {
            write_string_to_pointer(self.ptr, value)?;
            self.ptr = self.ptr.add(24);
        };
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    // Test:
    // * Side A writes to the FFI buffer
    // * Side B reads from it
    // * Side A frees it
    #[test]
    fn test_one_side_writes() {
        let ffi_buffer = ffi_buffer_alloc();
        // Side A writes
        let mut writer = unsafe { FfiBufferCursor::new(ffi_buffer) };
        writer.write_u8(1).unwrap();
        writer.write_f32(2.0).unwrap();
        writer.write_i32(-3).unwrap();
        writer.write_bool(true).unwrap();
        writer.write_string("test-data".into()).unwrap();
        let mut pointee = String::from("pointee");
        let pointer = pointee.as_mut_ptr();
        writer.write_ptr(pointer).unwrap();
        // Side B writes
        let mut reader = unsafe { FfiBufferCursor::new(ffi_buffer) };
        assert_eq!(reader.read_u8().unwrap(), 1);
        assert_eq!(reader.read_f32().unwrap(), 2.0);
        assert_eq!(reader.read_i32().unwrap(), -3);
        assert!(reader.read_bool().unwrap());
        assert_eq!(reader.read_string().unwrap(), "test-data");
        assert_eq!(reader.read_ptr().unwrap(), pointer);
        // Side A frees
        ffi_buffer_free(ffi_buffer);
    }

    // Test:
    // * Side A writes to the FFI buffer
    // * Side B reads from it
    // * Side B writes to it
    // * Side A reads from it
    // * Side A frees it
    #[test]
    fn test_both_sides_write() {
        let ffi_buffer = ffi_buffer_alloc();
        // Side A writes
        let mut writer = unsafe { FfiBufferCursor::new(ffi_buffer) };
        writer.write_u8(1).unwrap();
        writer.write_f32(2.0).unwrap();
        writer.write_i32(-3).unwrap();
        writer.write_bool(true).unwrap();
        writer.write_string("test-data".into()).unwrap();
        // Side B reads
        let mut reader = unsafe { FfiBufferCursor::new(ffi_buffer) };
        assert_eq!(reader.read_u8().unwrap(), 1);
        assert_eq!(reader.read_f32().unwrap(), 2.0);
        assert_eq!(reader.read_i32().unwrap(), -3);
        assert!(reader.read_bool().unwrap());
        assert_eq!(reader.read_string().unwrap(), "test-data");
        // Side B writes
        let mut writer = unsafe { FfiBufferCursor::new(ffi_buffer) };
        writer.write_i8(4).unwrap();
        writer.write_f64(5.0).unwrap();
        writer.write_string("test-data2".into()).unwrap();
        // Side A reads
        let mut reader = unsafe { FfiBufferCursor::new(ffi_buffer) };
        assert_eq!(reader.read_i8().unwrap(), 4);
        assert_eq!(reader.read_f64().unwrap(), 5.0);
        assert_eq!(reader.read_string().unwrap(), "test-data2");
        // Side A drops
        ffi_buffer_free(ffi_buffer);
    }

    // Like `test_one_side_writes`, but with enough data so that we need to allocate multiple
    // mini buffers.
    #[test]
    fn test_one_side_writes_large_alloc() {
        let ffi_buffer = ffi_buffer_alloc();
        // Side A writes
        let mut writer = unsafe { FfiBufferCursor::new(ffi_buffer) };
        for x in 0..250 {
            writer.write_u8(x).unwrap();
        }
        // Write a string now, where it's most awkward -- some of the fields will fit in the
        // buffer, but not all
        writer.write_string("test-data".into()).unwrap();
        for x in 0..250 {
            writer.write_u64(x).unwrap();
        }
        // Side B reads
        let mut reader = unsafe { FfiBufferCursor::new(ffi_buffer) };
        for x in 0..250 {
            assert_eq!(reader.read_u8().unwrap(), x);
        }
        assert_eq!(reader.read_string().unwrap(), "test-data");
        for x in 0..250 {
            assert_eq!(reader.read_u64().unwrap(), x);
        }
        // Side A frees
        ffi_buffer_free(ffi_buffer);
    }

    // Like `test_both_sides_write`, but with enough data so that we need to allocate multiple
    // mini buffers.
    #[test]
    fn test_both_sides_write_large_alloc() {
        let ffi_buffer = ffi_buffer_alloc();
        // Side A writes
        let mut writer = unsafe { FfiBufferCursor::new(ffi_buffer) };
        for x in 0..250 {
            writer.write_u8(x).unwrap();
        }
        writer.write_string("test-data".into()).unwrap();
        for x in 0..250 {
            writer.write_u64(x).unwrap();
        }
        // Side B reads
        let mut reader = unsafe { FfiBufferCursor::new(ffi_buffer) };
        for x in 0..250 {
            assert_eq!(reader.read_u8().unwrap(), x);
        }
        assert_eq!(reader.read_string().unwrap(), "test-data");
        for x in 0..250 {
            assert_eq!(reader.read_u64().unwrap(), x);
        }
        // Side B writes
        let mut writer = unsafe { FfiBufferCursor::new(ffi_buffer) };
        for x in 0..1000 {
            writer.write_f64(x as f64 / 2.0).unwrap();
        }
        // Side A reads
        let mut reader = unsafe { FfiBufferCursor::new(ffi_buffer) };
        for x in 0..1000 {
            assert_eq!(reader.read_f64().unwrap(), x as f64 / 2.0);
        }
        // Side A frees
        ffi_buffer_free(ffi_buffer);
    }
}
