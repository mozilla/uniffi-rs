/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// Like the `InfoFfi` trait from `ffi_support`, but local so that I can add some
// alternative implementations for different builtin types. Also so we can pass
// values in over the FFI as well as pass them back out.

use anyhow::Result;

pub unsafe trait ViaFfi: Sized {
  type Value;
  fn into_ffi_value(self) -> Self::Value;
  fn try_from_ffi_value(v: Self::Value) -> Result<Self>;
}

macro_rules! impl_via_ffi_for_primitive {
  ($($T:ty),+) => {$(
    unsafe impl ViaFfi for $T {
      type Value = Self;
      #[inline] fn into_ffi_value(self) -> Self { self }
      #[inline] fn try_from_ffi_value(v: Self::Value) -> Result<Self> { Ok(v) }
    }
  )+}
}

impl_via_ffi_for_primitive![(), i8, u8, i16, u16, i32, u32, i64, u64, f32, f64];