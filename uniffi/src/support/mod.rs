/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use anyhow::{bail, Result};
pub use bytes::buf::{Buf, BufMut};
use ffi_support::ByteBuffer;

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
