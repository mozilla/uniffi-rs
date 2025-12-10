/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::{Handle, RustBuffer, RustCallStatus, RustCallStatusCode};
use std::{mem::ManuallyDrop, ptr::NonNull};

/// Serialize a FFI value to a buffer
///
/// This is how the pointer FFI passes arguments.
/// It's similar, to the [crate::Lift::read] and [crate::Lower::write] methods, but implemented on the FFI types rather than Rust types:
///
/// - [crate::Lift] and [crate::Lower] are implemented on Rust types like String and user-defined records.
/// - [FfiSerialize] is implemented on the FFI types like [RustBuffer], [RustCallStatus], and vtable structs.
/// - All 3 traits are implemented for simple cases where the FFI type and Rust type are the same, for example numeric types.
/// - [FfiSerialize] adds padding so all elements have alignment 8 (64-bits).
/// - [FfiSerialize] uses a constant size to store each type and therefore each buffer has a static size, known at codegen time.
///
/// [FfiSerialize] is used to generate alternate scaffolding functions that simplify work needed to implement the bindings on the other side.
///
/// The pointer FFI version of the scaffolding functions:
///   - Input a pointer to a buffer large enough to store either all arguments or the return value plus a `RustCallStatus`.
///   - Read argument data from the buffer
///   - Calls the Rust function
///   - Writes the `RustCallStatus` then the return value to the buffer.
pub trait FfiSerialize: Sized {
    /// Bytes required to store this FFI type, this must be a multiple of 8.
    const SIZE: usize;

    /// Get a value from a ffi buffer
    ///
    /// Note: `buf` should be thought of as `&[u8; Self::SIZE]`, but it can't be spelled out that way
    /// since Rust doesn't support that usage of const generics yet.
    ///
    /// # Safety
    ///
    /// Reading from a FFI buffer must follow the pointer FFI protocol
    unsafe fn get(buf: &[u8]) -> Self;

    /// Put a value to a ffi buffer
    ///
    /// Note: `buf` should be thought of as `&[u8; Self::SIZE]`, but it can't be spelled out that way
    /// since Rust doesn't support that usage of const generics yet.
    ///
    /// # Safety
    ///
    /// Writing to a FFI buffer must follow the pointer FFI protocol
    unsafe fn put(buf: &mut [u8], value: Self);

    /// Read a value from a ffi buffer ref and advance it
    ///
    /// buf must have a length of at least `Self::Size`
    ///
    /// # Safety
    ///
    /// Reading from a FFI buffer must follow the pointer FFI protocol
    unsafe fn read(buf: &mut &[u8]) -> Self {
        let value = Self::get(buf);
        *buf = &buf[Self::SIZE..];
        value
    }

    /// Write a value to a ffi buffer ref and advance it
    ///
    /// buf must have a length of at least `Self::Size`
    ///
    /// # Safety
    ///
    /// Writing to a FFI buffer must follow the pointer FFI protocol
    unsafe fn write(buf: &mut &mut [u8], value: Self) {
        Self::put(buf, value);
        // Lifetime dance taken from `bytes::BufMut`
        let (_, new_buf) = ::core::mem::take(buf).split_at_mut(Self::SIZE);
        *buf = new_buf;
    }
}

/// Get the FFI buffer size for a function call
///
/// There's two forms:
///   - Single list of types, returns the sum of all sizes
///   - Two lists, returns the max sum.  This is used to create buffers big enough to hold either
///     all arguments or all return values.
#[macro_export]
macro_rules! ffi_buffer_size {
    (($($TY:ty),*) $(,)?) => {
        const {
            0
            $(
                + <$TY as $crate::FfiSerialize>::SIZE
            )*
        }
    };
    (($($ARG_TY:ty),*), ($($RETURN_TY:ty),*) $(,)?) => {
        const {
            let arg_size = (
                0
                $(
                    + <$ARG_TY as $crate::FfiSerialize>::SIZE
                )*
            );
            let return_size = (
                0
                $(
                    + <$RETURN_TY as $crate::FfiSerialize>::SIZE
                )*
            );
            if (arg_size >= return_size) {
                arg_size
            } else {
                return_size
            }
        }
    };
}

macro_rules! define_ffi_serialize_simple_cases {
    ($($T:ty),* $(,)?) => {
        $(
            impl FfiSerialize for $T {
                const SIZE: usize = 8;

                unsafe fn get(buf: &[u8]) -> Self {
                    let buf: &[Self] = ::std::mem::transmute(buf);
                    buf[0]
                }

                unsafe fn put(buf: &mut[u8], value: Self) {
                    let buf: &mut[Self] = ::std::mem::transmute(buf);
                    buf[0] = value;
                }
            }
        )*
    };
}

define_ffi_serialize_simple_cases! {
    i8,
    u8,
    i16,
    u16,
    i32,
    u32,
    i64,
    u64,
    f32,
    f64,
    bool,
}
impl FfiSerialize for *const std::ffi::c_void {
    const SIZE: usize = 8;

    unsafe fn get(buf: &[u8]) -> Self {
        let buf: &[u64] = ::std::mem::transmute(buf);
        buf[0] as Self
    }

    unsafe fn put(buf: &mut [u8], value: Self) {
        let buf: &mut [u64] = ::std::mem::transmute(buf);
        buf[0] = value as u64
    }
}

impl FfiSerialize for () {
    const SIZE: usize = 0;

    unsafe fn get(_buf: &[u8]) -> Self {}

    unsafe fn put(_buf: &mut [u8], _value: Self) {}
}

impl<T> FfiSerialize for NonNull<T> {
    const SIZE: usize = 8;

    unsafe fn get(buf: &[u8]) -> Self {
        let buf: &[u64] = ::std::mem::transmute(buf);
        NonNull::new_unchecked(buf[0] as *mut T)
    }

    unsafe fn put(buf: &mut [u8], value: Self) {
        let buf: &mut [u64] = ::std::mem::transmute(buf);
        buf[0] = value.as_ptr() as u64
    }
}

impl FfiSerialize for Handle {
    const SIZE: usize = 8;

    unsafe fn get(buf: &[u8]) -> Self {
        let buf: &[u64] = ::std::mem::transmute(buf);
        Handle::from_raw_unchecked(buf[0])
    }

    unsafe fn put(buf: &mut [u8], value: Self) {
        let buf: &mut [u64] = ::std::mem::transmute(buf);
        buf[0] = value.as_raw()
    }
}

impl FfiSerialize for RustBuffer {
    const SIZE: usize = 24;

    unsafe fn get(buf: &[u8]) -> Self {
        let buf: &[u64] = ::std::mem::transmute(buf);
        let capacity: u64 = buf[0];
        let len: u64 = buf[1];
        let data: *mut u8 = buf[2] as *mut u8;
        RustBuffer::from_raw_parts(data, len, capacity)
    }

    unsafe fn put(buf: &mut [u8], value: Self) {
        let buf: &mut [u64] = ::std::mem::transmute(buf);
        buf[0] = value.capacity;
        buf[1] = value.len;
        buf[2] = value.data as u64;
    }
}

impl FfiSerialize for RustCallStatus {
    const SIZE: usize = 32;

    unsafe fn get(buf: &[u8]) -> Self {
        let code = <i8 as FfiSerialize>::get(buf);
        let error_buf = <RustBuffer as FfiSerialize>::get(&buf[8..]);
        Self {
            code: RustCallStatusCode::try_from(code).unwrap_or(RustCallStatusCode::UnexpectedError),
            error_buf: ManuallyDrop::new(error_buf),
        }
    }

    unsafe fn put(buf: &mut [u8], value: Self) {
        <i8 as FfiSerialize>::put(buf, value.code as i8);
        <RustBuffer as FfiSerialize>::put(&mut buf[8..], ManuallyDrop::into_inner(value.error_buf));
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{Handle, RustBuffer, RustCallStatus, RustCallStatusCode};

    #[test]
    fn test_ffi_buffer_size() {
        // Single list of types.  `ffi_buffer_size` should return the sum of the sizes
        assert_eq!(
            ffi_buffer_size!((u8, f32, bool, Handle, (), RustBuffer)),
            56
        );
        // Two list of types. `ffi_buffer_size` should return the max sum.
        // This is how we create a buffer big enough to hold either the arguments and return values.
        assert_eq!(ffi_buffer_size!((u8, Handle), (RustBuffer)), 24);
    }

    #[test]
    fn test_ffi_serialize() {
        let mut some_data = vec![1, 2, 3];
        let void_ptr = some_data.as_mut_ptr() as *const std::ffi::c_void;
        let rust_buffer = unsafe { RustBuffer::from_raw_parts(some_data.as_mut_ptr(), 2, 3) };
        let orig_rust_buffer_data = (
            rust_buffer.data_pointer(),
            rust_buffer.len(),
            rust_buffer.capacity(),
        );
        let handle = unsafe { Handle::from_raw(101).unwrap() };
        let rust_call_status = RustCallStatus::default();
        let rust_call_status_error_buf = &rust_call_status.error_buf;
        let orig_rust_call_status_buffer_data = (
            rust_call_status_error_buf.data_pointer(),
            rust_call_status_error_buf.len(),
            rust_call_status_error_buf.capacity(),
        );
        let mut buf = [u8::default(); 21 * 8];
        let mut buf_writer = buf.as_mut_slice();
        unsafe {
            <u8 as FfiSerialize>::write(&mut buf_writer, 0);
            <i8 as FfiSerialize>::write(&mut buf_writer, 1);
            <u16 as FfiSerialize>::write(&mut buf_writer, 2);
            <i16 as FfiSerialize>::write(&mut buf_writer, 3);
            <u32 as FfiSerialize>::write(&mut buf_writer, 4);
            <i32 as FfiSerialize>::write(&mut buf_writer, 5);
            <u64 as FfiSerialize>::write(&mut buf_writer, 6);
            <i64 as FfiSerialize>::write(&mut buf_writer, 7);
            <f32 as FfiSerialize>::write(&mut buf_writer, 0.1);
            <f64 as FfiSerialize>::write(&mut buf_writer, 0.2);
            <bool as FfiSerialize>::write(&mut buf_writer, true);
            <*const std::ffi::c_void as FfiSerialize>::write(&mut buf_writer, void_ptr);
            <RustBuffer as FfiSerialize>::write(&mut buf_writer, rust_buffer);
            <RustCallStatus as FfiSerialize>::write(&mut buf_writer, rust_call_status);
            <Handle as FfiSerialize>::write(&mut buf_writer, handle.clone());
            #[allow(clippy::needless_borrows_for_generic_args)]
            <() as FfiSerialize>::write(&mut buf_writer, ());
        }

        let mut buf_reader = buf.as_slice();
        unsafe {
            assert_eq!(<u8 as FfiSerialize>::read(&mut buf_reader), 0);
            assert_eq!(<i8 as FfiSerialize>::read(&mut buf_reader), 1);
            assert_eq!(<u16 as FfiSerialize>::read(&mut buf_reader), 2);
            assert_eq!(<i16 as FfiSerialize>::read(&mut buf_reader), 3);
            assert_eq!(<u32 as FfiSerialize>::read(&mut buf_reader), 4);
            assert_eq!(<i32 as FfiSerialize>::read(&mut buf_reader), 5);
            assert_eq!(<u64 as FfiSerialize>::read(&mut buf_reader), 6);
            assert_eq!(<i64 as FfiSerialize>::read(&mut buf_reader), 7);
            assert_eq!(<f32 as FfiSerialize>::read(&mut buf_reader), 0.1);
            assert_eq!(<f64 as FfiSerialize>::read(&mut buf_reader), 0.2);
            assert!(<bool as FfiSerialize>::read(&mut buf_reader));
            assert_eq!(
                <*const std::ffi::c_void as FfiSerialize>::read(&mut buf_reader),
                void_ptr
            );
            let rust_buffer2 = <RustBuffer as FfiSerialize>::read(&mut buf_reader);
            assert_eq!(
                (
                    rust_buffer2.data_pointer(),
                    rust_buffer2.len(),
                    rust_buffer2.capacity()
                ),
                orig_rust_buffer_data,
            );

            let rust_call_status2 = <RustCallStatus as FfiSerialize>::read(&mut buf_reader);
            assert_eq!(rust_call_status2.code, RustCallStatusCode::Success);

            let rust_call_status2_error_buf = ManuallyDrop::into_inner(rust_call_status2.error_buf);
            assert_eq!(
                (
                    rust_call_status2_error_buf.data_pointer(),
                    rust_call_status2_error_buf.len(),
                    rust_call_status2_error_buf.capacity(),
                ),
                orig_rust_call_status_buffer_data
            );
            assert_eq!(<Handle as FfiSerialize>::read(&mut buf_reader), handle);
            // Ensure that `read` with a unit struct doesn't panic.  No need to assert anything, since
            // the return type is ().
            <() as FfiSerialize>::read(&mut buf_reader);
        }
    }
}
