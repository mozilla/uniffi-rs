/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use anyhow::{bail, Result};
use bytes::buf::{Buf, BufMut};
use ffi_support::ByteBuffer;
use paste::paste;
use std::convert::TryFrom;
use std::ffi::CString;

// It would be nice if this module was behind a cfg(test) guard, but it
// doesn't work between crates so let's hope LLVM tree-shaking works well.
pub mod testing;

// Re-export the libs that we use in the generated code,
// so the consumer doesn't have to depend on them directly.
pub mod deps {
    pub use anyhow;
    pub use bytes;
    pub use ffi_support;
    pub use lazy_static;
    pub use log;
}

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

pub fn check_remaining<B: Buf>(buf: &B, num_bytes: usize) -> Result<()> {
    if buf.remaining() < num_bytes {
        bail!("not enough bytes remaining in buffer");
    }
    Ok(())
}

pub trait Liftable: Sized {
    fn try_lift_from<B: Buf>(buf: &mut B) -> Result<Self>;
}

pub fn try_lift<T: Liftable>(buf: ByteBuffer) -> Result<T> {
    let vec = buf.destroy_into_vec();
    let mut buf = vec.as_slice();
    let value = <T as Liftable>::try_lift_from(&mut buf)?;
    if buf.remaining() != 0 {
        bail!("junk data left in buffer after deserializing")
    }
    Ok(value)
}

macro_rules! impl_lowerable_liftable_for_num_primitive {
    ($($T:ty,)+) => { impl_lowerable_liftable_for_num_primitive!($($T),+); };
    ($($T:ty),*) => {
            $(
                paste! {
                    impl Liftable for $T {
                        fn try_lift_from<B: Buf>(buf: &mut B) -> Result<Self> {
                            check_remaining(buf, std::mem::size_of::<$T>())?;
                            Ok(buf.[<get_ $T>]())
                        }
                    }
                    impl Lowerable for $T {
                        fn lower_into<B: BufMut>(&self, buf: &mut B) {
                            buf.[<put_ $T>](*self);
                        }
                    }
                }
            )*
    };
}

impl_lowerable_liftable_for_num_primitive! {
    i8, u8, i16, u16, i32, u32, i64, u64, f32, f64
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

/// Support for passing Strings back and forth across the FFI.
///
/// Unlike many other implementations of `ViaFfi`, this passes a pointer rather
/// than copying the data from one side to the other. This is a safety hazard,
/// but turns out to be pretty nice for useability.
///
/// (In practice, we do end up copying the data, the copying just happens on
/// the foreign language side rather than here in the rust code.)
unsafe impl ViaFfi for String {
    type Value = *mut std::os::raw::c_char;

    // This returns a raw pointer to the underlying bytes, so it's very important
    // that it consume ownership of the String, which is relinquished to the foreign
    // language code (and can be returned by it passing the pointer back).
    fn into_ffi_value(self) -> Self::Value {
        ffi_support::rust_string_to_c(self)
    }

    // The argument here *must* be a uniquely-owned pointer previously obtained
    // from `info_ffi_value` above. It will try to panic if you give it an invalid
    // pointer, but there's no guarantee that it will.
    fn try_from_ffi_value(v: Self::Value) -> Result<Self> {
        if v.is_null() {
            bail!("null pointer passed as String")
        }
        let cstr = unsafe { CString::from_raw(v) };
        // This turns the buffer back into a `String` without copying the data
        // and without re-checking it for validity of the utf8. If the pointer
        // came from a valid String then there's no point in re-checking the utf8,
        // and if it didn't then bad things are going to happen regardless of
        // whether we check for valid utf8 data or not.
        Ok(unsafe { String::from_utf8_unchecked(cstr.into_bytes()) })
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
