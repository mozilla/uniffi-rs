/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use anyhow::{bail, Result};
pub use bytes::buf::{Buf, BufMut};
use ffi_support::ByteBuffer;
use std::convert::TryFrom;

pub mod tests;

/// Any type that can be returned over the FFI must implement the `Lowerable` trait, to define how
/// it gets lowered into bytes for transit. We provide default implementtions for primitive types,
/// and a typical implementation for composite types would lower each member in turn.

pub trait Lowerable: Sized {
    fn lower_into<B: BufMut>(&self, buf: &mut B);
}

pub fn lower<T: Lowerable>(value: T) -> ByteBuffer {
    let mut buf = Vec::new();
    Lowerable::lower_into(&value, &mut buf);
    ByteBuffer::from_vec(buf)
}

impl<T: Lowerable> Lowerable for &T {
    fn lower_into<B: BufMut>(&self, buf: &mut B) {
        (*self).lower_into(buf);
    }
}

impl Lowerable for u32 {
    fn lower_into<B: BufMut>(&self, buf: &mut B) {
        buf.put_u32(*self);
    }
}

impl Lowerable for f64 {
    fn lower_into<B: BufMut>(&self, buf: &mut B) {
        buf.put_f64(*self);
    }
}

impl<T: Lowerable> Lowerable for Option<T> {
    fn lower_into<B: BufMut>(&self, buf: &mut B) {
        match self {
            None => buf.put_u8(0),
            Some(v) => {
                buf.put_u8(1);
                v.lower_into(buf);
            }
        }
    }
}

impl<T: Lowerable> Lowerable for Vec<T> {
    fn lower_into<B: BufMut>(&self, buf: &mut B) {
        let len = u32::try_from(self.len()).unwrap();
        buf.put_u32(len); // We limit arrays to u32::MAX bytes
        for item in self.iter() {
            item.lower_into(buf);
        }
    }
}

/// Any type that can be received over the FFI must implement the `Liftable` trait, to define how
/// it gets lifted from bytes into a useable Rust value. We provide default implementtions for
/// primitive types, and a typical implementation for composite types would lift each member in turn.

fn check_remaining<B: Buf>(buf: &B, num_bytes: usize) -> Result<()> {
    if buf.remaining() < num_bytes {
        bail!("not enough bytes remaining in buffer");
    }
    Ok(())
}

pub trait Liftable: Sized {
    fn try_lift_from<B: Buf>(buf: &mut B) -> Result<Self>;
}

pub fn try_lift<T: Liftable>(buf: ByteBuffer) -> Result<T> {
    let vec = buf.into_vec();
    let mut buf = vec.as_slice();
    let value = <T as Liftable>::try_lift_from(&mut buf)?;
    if buf.remaining() != 0 {
        bail!("junk data left in buffer after deserializing")
    }
    Ok(value)
}

impl Liftable for u32 {
    fn try_lift_from<B: Buf>(buf: &mut B) -> Result<Self> {
        check_remaining(buf, 4)?;
        Ok(buf.get_u32())
    }
}

impl Liftable for f64 {
    fn try_lift_from<B: Buf>(buf: &mut B) -> Result<Self> {
        check_remaining(buf, 8)?;
        Ok(buf.get_f64())
    }
}

impl<T: Liftable> Liftable for Option<T> {
    fn try_lift_from<B: Buf>(buf: &mut B) -> Result<Self> {
        check_remaining(buf, 1)?;
        Ok(match buf.get_u8() {
            0 => None,
            1 => Some(T::try_lift_from(buf)?),
            _ => bail!("unexpected tag byte for Option"),
        })
    }
}

impl<T: Liftable> Liftable for Vec<T> {
    fn try_lift_from<B: Buf>(buf: &mut B) -> Result<Self> {
        check_remaining(buf, 4)?;
        let len = buf.get_u32();
        let mut vec = Vec::with_capacity(len as usize);
        for _ in 0..len {
            vec.push(T::try_lift_from(buf)?)
        }
        Ok(vec)
    }
}

// The `ViaFfi` trait defines how to receive a type as an argument over the FFI,
// and how to return it from FFI functions. It allows us to implement a more efficient
// calling strategy than always lifting/lowering all types to a byte buffer.
//
// Types that cannot be passed via any more clever mechanism can instead get choose to
// `impl ViaFfiUsingByteBuffer`, which provides a default implementation that uses their
// `Lowerable` and `Liftable` impls to pass data by serializing in a ByteBuffer.
// Unlike the base `ViaFfi` trit, `ViaFfiUsingByteBuffer` is safe.
//
// (This trait is Like the `InfoFfi` trait from `ffi_support`, but local to this crate
// so that we can add some alternative implementations for different builtin types,
// and so that we can add support for receiving as well as returning).

pub unsafe trait ViaFfi: Sized {
    type Value;
    fn into_ffi_value(self) -> Self::Value;
    fn try_from_ffi_value(v: Self::Value) -> Result<Self>;
}

macro_rules! impl_via_ffi_for_primitive {
  ($($T:ty),+) => {$(
    unsafe impl ViaFfi for $T {
      type Value = Self;
      #[inline] fn into_ffi_value(self) -> Self::Value { self }
      #[inline] fn try_from_ffi_value(v: Self::Value) -> Result<Self> { Ok(v) }
    }
  )+}
}

impl_via_ffi_for_primitive![(), i8, u8, i16, u16, i32, u32, i64, u64, f32, f64];

pub trait ViaFfiUsingByteBuffer: Liftable + Lowerable {}

unsafe impl<T: ViaFfiUsingByteBuffer> ViaFfi for T {
    type Value = ffi_support::ByteBuffer;
    #[inline]
    fn into_ffi_value(self) -> Self::Value {
        lower(&self)
    }
    #[inline]
    fn try_from_ffi_value(v: Self::Value) -> anyhow::Result<Self> {
        try_lift(v)
    }
}

unsafe impl<T: Liftable + Lowerable> ViaFfi for Option<T> {
    type Value = ffi_support::ByteBuffer;
    #[inline]
    fn into_ffi_value(self) -> Self::Value {
        lower(&self)
    }
    #[inline]
    fn try_from_ffi_value(v: Self::Value) -> anyhow::Result<Self> {
        try_lift(v)
    }
}

unsafe impl<T: Liftable + Lowerable> ViaFfi for Vec<T> {
    type Value = ffi_support::ByteBuffer;
    #[inline]
    fn into_ffi_value(self) -> Self::Value {
        lower(&self)
    }
    #[inline]
    fn try_from_ffi_value(v: Self::Value) -> anyhow::Result<Self> {
        try_lift(v)
    }
}

unsafe impl<'a> ViaFfi for &'a str {
    type Value = ffi_support::FfiStr<'a>;
    fn try_from_ffi_value(v: Self::Value) -> Result<Self> {
        Ok(v.as_str())
    }

    // This should never happen since there is asymmetry with strings
    // We typically go FfiStr -> &str for argumetns and
    // String -> *mut c_char for return values.
    // We panic for now, the ViaFfi triat will be
    // broken down to IntoFfi and TryFromFfi to prevent us from having
    // to add this
    fn into_ffi_value(self) -> Self::Value {
        panic!("Invalid conversion. into_ffi_value should not be called on a &str, a String -> *mut c_char conversion should be used instead.")
    }
}

unsafe impl ViaFfi for String {
    type Value = *mut std::os::raw::c_char;
    fn into_ffi_value(self) -> Self::Value {
        ffi_support::rust_string_to_c(self)
    }

    // This should never happen since there is asymmetry with strings
    // We typically go FfiStr -> &str for argumetns and
    // String -> *mut c_char for return values.
    // We panic for now, the ViaFfi triat will be
    // broken down to IntoFfi and TryFromFfi to prevent us from having
    // to add this
    fn try_from_ffi_value(_: Self::Value) -> Result<Self> {
        panic!("Invalid conversion. try_from_ffi_value should not be called on a c_char, an FfiStr<'_> -> &str conversion should be used instead.")
    }
}

impl Lowerable for String {
    fn lower_into<B: BufMut>(&self, buf: &mut B) {
        let len = u32::try_from(self.len()).unwrap();
        buf.put_u32(len); // We limit strings to u32::MAX bytes
        buf.put(self.as_bytes());
    }
}

// Having this be for &str instead of String would have been nice for consistency but...
// it seems very dangerous and only possible with some unsafe magic
// and having a slice referencing the buffer seems like a recipie for
// disaster... so we have a String here instead.
impl Liftable for String {
    fn try_lift_from<B: Buf>(buf: &mut B) -> Result<Self> {
        check_remaining(buf, 4)?;
        let len = buf.get_u32();
        check_remaining(buf, len as usize)?;
        let bytes = &buf.bytes()[..len as usize];
        let res = String::from_utf8(bytes.to_vec())?;
        buf.advance(len as usize);
        Ok(res)
    }
}

unsafe impl ViaFfi for bool {
    type Value = u8;
    fn into_ffi_value(self) -> Self::Value {
        if self {
            1
        } else {
            0
        }
    }
    fn try_from_ffi_value(v: Self::Value) -> Result<Self> {
        Ok(match v {
            0 => false,
            1 => true,
            _ => bail!("unexpected byte for Boolean"),
        })
    }
}

impl Lowerable for bool {
    fn lower_into<B: BufMut>(&self, buf: &mut B) {
        buf.put_u8(ViaFfi::into_ffi_value(*self));
    }
}

impl Liftable for bool {
    fn try_lift_from<B: Buf>(buf: &mut B) -> Result<Self> {
        check_remaining(buf, 1)?;
        ViaFfi::try_from_ffi_value(buf.get_u8())
    }
}
