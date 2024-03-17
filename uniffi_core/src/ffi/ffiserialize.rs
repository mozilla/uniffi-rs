/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::ptr::NonNull;

/// Serialize a FFI value to a buffer
///
/// This trait allows FFI types to be read/written from buffers.
/// It's similar, to the [crate::Lift::read] and [crate::Lower::write] methods, but implemented on the FFI types rather than Rust types.
/// It's useful to compare the two:
///
/// - [crate::Lift] and [crate::Lower] are implemented on Rust types like String and user-defined records.
/// - [FfiSerialize] is implemented on the FFI types like RustBuffer, RustCallStatus, and vtable structs.
/// - All 3 traits are implemented for simple cases where the FFI type and Rust type are the same, for example numeric types..
/// - [FfiSerialize] uses u64 elements rather than u8 elements.  This creates better alignment of the arguments at the cost of extra size.
/// - [FfiSerialize] uses a constant size to store each type.
///
/// [FfiSerialize] is used to generate alternate forms of the scaffolding functions that input two pointers to `u64` buffers.
/// One is used to read the arguments from, one to write the return value to.
///
/// This is currently only used in the gecko-js bindings for Firefox, but could maybe be useful for other external bindings.
pub trait FfiSerialize: Sized {
    /// Number of u64 items required to store this FFI type
    const SIZE: usize;

    /// Get a value from a u64 buffer
    ///
    /// Note: buf should be thought of as &[u64: Self::SIZE], but it can't be spelled out that way
    /// since Rust doesn't support that usage of const generics yet.
    fn get(buf: &[u64]) -> Self;

    /// Put a value to a u64 buffer
    ///
    /// Note: buf should be thought of as &[u64: Self::SIZE], but it can't be spelled out that way
    /// since Rust doesn't support that usage of const generics yet.
    fn put(buf: &mut [u64], value: Self);

    /// Read a value from a u64 buffer ref and advance it
    fn read(buf: &mut &[u64]) -> Self {
        let value = Self::get(buf);
        *buf = &buf[Self::SIZE..];
        value
    }

    /// Write a value to a u64 buffer ref and advance it
    fn write(buf: &mut &mut [u64], value: Self) {
        Self::put(buf, value);
        // Lifetime dance taken from `bytes::BufMut`
        let (_, new_buf) = core::mem::take(buf).split_at_mut(Self::SIZE);
        *buf = new_buf;
    }
}

/// Get the FFI buffer size for list of types
#[macro_export]
macro_rules! ffi_buffer_size {
    ($($T:ty),* $(,)?) => {
        (
            0
            $(
                + <$T as $crate::FfiSerialize>::SIZE
            )*
        )
    }
}

macro_rules! define_ffi_serialize_simple_cases {
    ($($T:ty),* $(,)?) => {
        $(
            impl FfiSerialize for $T {
                const SIZE: usize = 1;

                fn get(buf: &[u64]) -> Self {
                    buf[0] as Self
                }

                fn put(buf: &mut[u64], value: Self) {
                    buf[0] = value as u64
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
    *const std::ffi::c_void,
}

impl FfiSerialize for f64 {
    const SIZE: usize = 1;

    fn get(buf: &[u64]) -> Self {
        f64::from_bits(buf[0])
    }

    fn put(buf: &mut [u64], value: Self) {
        buf[0] = f64::to_bits(value)
    }
}

impl FfiSerialize for f32 {
    const SIZE: usize = 1;

    fn get(buf: &[u64]) -> Self {
        f32::from_bits(buf[0] as u32)
    }

    fn put(buf: &mut [u64], value: Self) {
        buf[0] = f32::to_bits(value) as u64
    }
}

impl FfiSerialize for bool {
    const SIZE: usize = 1;

    fn get(buf: &[u64]) -> Self {
        buf[0] == 1
    }

    fn put(buf: &mut [u64], value: Self) {
        buf[0] = value as u64
    }
}

impl FfiSerialize for () {
    const SIZE: usize = 0;

    fn get(_buf: &[u64]) -> Self {}

    fn put(_buf: &mut [u64], _value: Self) {}
}

impl<T> FfiSerialize for &T {
    const SIZE: usize = 1;

    fn get(buf: &[u64]) -> Self {
        // Safety: this relies on the foreign code passing us valid pointers
        unsafe { &*(buf[0] as *const T) }
    }

    fn put(buf: &mut [u64], value: Self) {
        buf[0] = value as *const T as u64
    }
}

impl<T> FfiSerialize for &mut T {
    const SIZE: usize = 1;

    fn get(buf: &[u64]) -> Self {
        // Safety: this relies on the foreign code passing us valid pointers
        unsafe { &mut *(buf[0] as *mut T) }
    }

    fn put(buf: &mut [u64], value: Self) {
        buf[0] = value as *mut T as u64
    }
}

impl<T> FfiSerialize for NonNull<T> {
    const SIZE: usize = 1;

    fn get(buf: &[u64]) -> Self {
        // Safety: this relies on the foreign code passing us valid pointers
        unsafe { &mut *(buf[0] as *mut T) }.into()
    }

    fn put(buf: &mut [u64], value: Self) {
        buf[0] = value.as_ptr() as u64
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{Handle, RustBuffer, RustCallStatus, RustCallStatusCode};

    #[test]
    fn test_ffi_buffer_size() {
        assert_eq!(ffi_buffer_size!(u8), 1);
        assert_eq!(ffi_buffer_size!(i8), 1);
        assert_eq!(ffi_buffer_size!(u16), 1);
        assert_eq!(ffi_buffer_size!(i16), 1);
        assert_eq!(ffi_buffer_size!(u32), 1);
        assert_eq!(ffi_buffer_size!(i32), 1);
        assert_eq!(ffi_buffer_size!(u64), 1);
        assert_eq!(ffi_buffer_size!(i64), 1);
        assert_eq!(ffi_buffer_size!(f32), 1);
        assert_eq!(ffi_buffer_size!(f64), 1);
        assert_eq!(ffi_buffer_size!(bool), 1);
        assert_eq!(ffi_buffer_size!(*const std::ffi::c_void), 1);
        assert_eq!(ffi_buffer_size!(RustBuffer), 3);
        assert_eq!(ffi_buffer_size!(RustCallStatus), 4);
        assert_eq!(ffi_buffer_size!(Handle), 1);
        assert_eq!(ffi_buffer_size!(&u8), 1);
        assert_eq!(ffi_buffer_size!(()), 0);

        assert_eq!(ffi_buffer_size!(u8, f32, bool, Handle, (), RustBuffer), 7);
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
        let handle = Handle::from_raw(101).unwrap();
        let rust_call_status = RustCallStatus::new();
        let rust_call_status_error_buf = unsafe { rust_call_status.error_buf.assume_init_ref() };
        let orig_rust_call_status_buffer_data = (
            rust_call_status_error_buf.data_pointer(),
            rust_call_status_error_buf.len(),
            rust_call_status_error_buf.capacity(),
        );
        let f64_value = 2.5;
        let mut buf = [0_u64; 21];
        let mut buf_writer = buf.as_mut_slice();
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
        <Handle as FfiSerialize>::write(&mut buf_writer, handle);
        #[allow(clippy::needless_borrow)]
        <&f64 as FfiSerialize>::write(&mut buf_writer, &f64_value);
        <() as FfiSerialize>::write(&mut buf_writer, ());

        let mut buf_reader = buf.as_slice();
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

        let rust_call_status2_error_buf = unsafe { rust_call_status2.error_buf.assume_init() };
        assert_eq!(
            (
                rust_call_status2_error_buf.data_pointer(),
                rust_call_status2_error_buf.len(),
                rust_call_status2_error_buf.capacity(),
            ),
            orig_rust_call_status_buffer_data
        );
        assert_eq!(<Handle as FfiSerialize>::read(&mut buf_reader), handle);
        assert_eq!(<&f64 as FfiSerialize>::read(&mut buf_reader), &f64_value);
        // Ensure that `read` with a unit struct doesn't panic.  No need to assert anything, since
        // the return type is ().
        <() as FfiSerialize>::read(&mut buf_reader);
    }
}
