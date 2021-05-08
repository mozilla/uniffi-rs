/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! # Runtime support code for uniffi
//!
//! This crate provides the small amount of runtime code that is required by the generated uniffi
//! component scaffolding in order to transfer data back and forth across the C-style FFI layer,
//! as well as some utilities for testing the generated bindings.
//!
//! The key concept here is the [`ViaFfi`] trait, which must be implemented for any type that can
//! be passed across the FFI, and which determines:
//!
//!  * How to [represent](ViaFfi::Value) values of that type in the low-level C-style type
//!    system of the FFI layer.
//!  * How to ["lower"](ViaFfi::lower) rust values of that type into an appropriate low-level
//!    FFI value.
//!  * How to ["lift"](ViaFfi::lift) low-level FFI values back into rust values of that type.
//!  * How to [write](ViaFfi::write) rust values of that type into a buffer, for cases
//!    where they are part of a compound data structure that is serialized for transfer.
//!  * How to [read](ViaFfi::read) rust values of that type from buffer, for cases
//!    where they are received as part of a compound data structure that was serialized for transfer.
//!
//! This logic encapsulates the rust-side handling of data transfer. Each foreign-language binding
//! must also implement a matching set of data-handling rules for each data type.
//!
//! In addition to the core` ViaFfi` trait, we provide a handful of struct definitions useful
//! for passing core rust types over the FFI, such as [`RustBuffer`].

use anyhow::{bail, Result};
use bytes::buf::{Buf, BufMut};
use paste::paste;
use std::{collections::HashMap, convert::TryFrom};

pub mod ffi;
pub use ffi::*;

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
    pub use static_assertions;
}

const PACKAGE_VERSION: &str = env!("CARGO_PKG_VERSION");

// For the significance of this magic number 10 here, and the reason that
// it can't be a named constant, see the `check_compatible_version` function.
static_assertions::const_assert!(PACKAGE_VERSION.as_bytes().len() < 10);

/// Check whether the uniffi runtime version is compatible a given uniffi_bindgen version.
///
/// The result of this check may be used to ensure that generated Rust scaffolding is
/// using a compatible version of the uniffi runtime crate. It's a `const fn` so that it
/// can be used to perform such a check at compile time.
#[allow(clippy::len_zero)]
pub const fn check_compatible_version(bindgen_version: &'static str) -> bool {
    // While UniFFI is still under heavy development, we require that
    // the runtime support crate be precisely the same version as the
    // build-time bindgen crate.
    //
    // What we want to achieve here is checking two strings for equality.
    // Unfortunately Rust doesn't yet support calling the `&str` equals method
    // in a const context. We can hack around that by doing a byte-by-byte
    // comparison of the underlying bytes.
    let package_version = PACKAGE_VERSION.as_bytes();
    let bindgen_version = bindgen_version.as_bytes();
    // What we want to achieve here is a loop over the underlying bytes,
    // something like:
    // ```
    //  if package_version.len() != bindgen_version.len() {
    //      return false
    //  }
    //  for i in 0..package_version.len() {
    //      if package_version[i] != bindgen_version[i] {
    //          return false
    //      }
    //  }
    //  return true
    // ```
    // Unfortunately stable Rust doesn't allow `if` or `for` in const contexts,
    // so code like the above would only work in nightly. We can hack around it by
    // statically asserting that the string is shorter than a certain length
    // (currently 10 bytes) and then manually unrolling that many iterations of the loop.
    //
    // Yes, I am aware that this is horrific, but the externally-visible
    // behaviour is quite nice for consumers!
    package_version.len() == bindgen_version.len()
        && (package_version.len() == 0 || package_version[0] == bindgen_version[0])
        && (package_version.len() <= 1 || package_version[1] == bindgen_version[1])
        && (package_version.len() <= 2 || package_version[2] == bindgen_version[2])
        && (package_version.len() <= 3 || package_version[3] == bindgen_version[3])
        && (package_version.len() <= 4 || package_version[4] == bindgen_version[4])
        && (package_version.len() <= 5 || package_version[5] == bindgen_version[5])
        && (package_version.len() <= 6 || package_version[6] == bindgen_version[6])
        && (package_version.len() <= 7 || package_version[7] == bindgen_version[7])
        && (package_version.len() <= 8 || package_version[8] == bindgen_version[8])
        && (package_version.len() <= 9 || package_version[9] == bindgen_version[9])
        && package_version.len() < 10
}

/// Assert that the uniffi runtime version matches an expected value.
///
/// This is a helper hook for the generated Rust scaffolding, to produce a compile-time
/// error if the version of `uniffi_bindgen` used to generate the scaffolding was
/// incompatible with the version of `uniffi` being used at runtime.
#[macro_export]
macro_rules! assert_compatible_version {
    ($v:expr $(,)?) => {
        uniffi::deps::static_assertions::const_assert!(uniffi::check_compatible_version($v));
    };
}

/// Trait defining how to transfer values via the FFI layer.
///
/// The `ViaFfi` trait defines how to pass values of a particular type back-and-forth over
/// the uniffi generated FFI layer, both as standalone argument or return values, and as
/// part of serialized compound data structures.
///
/// (This trait is Like the `InfoFfi` trait from `ffi_support`, but local to this crate
/// so that we can add some alternative implementations for different builtin types,
/// and so that we can add support for receiving as well as returning).
///
/// ## Safety
///
/// This is an unsafe trait (implementing it requires `unsafe impl`) because we can't guarantee
/// that it's safe to pass your type out to foreign-language code and back again. Buggy
/// implementations of this trait might violate some assumptions made by the generated code,
/// or might not match with the corresponding code in the generated foreign-language bindings.
///
/// In general, you should not need to implement this trait by hand, and should instead rely on
/// implementations generated from your component UDL via the `uniffi-bindgen scaffolding` command.

pub unsafe trait ViaFfi: Sized {
    /// The low-level type used for passing values of this type over the FFI.
    ///
    /// This must be a C-compatible type (e.g. a numeric primitive, a `#[repr(C)]` struct) into
    /// which values of the target rust type can be converted.
    ///
    /// For complex data types, we currently recommend using `RustBuffer` and serializing
    /// the data for transfer. In theory it could be possible to build a matching
    /// `#[repr(C)]` struct for a complex data type and pass that instead, but explicit
    /// serialization is simpler and safer as a starting point.
    type FfiType;

    /// Lower a rust value of the target type, into an FFI value of type Self::FfiType.
    ///
    /// This trait method is used for sending data from rust to the foreign language code,
    /// by (hopefully cheaply!) converting it into someting that can be passed over the FFI
    /// and reconstructed on the other side.
    ///
    /// Note that this method takes an owned `self`; this allows it to transfer ownership
    /// in turn to the foreign language code, e.g. by boxing the value and passing a pointer.
    fn lower(self) -> Self::FfiType;

    /// Lift a rust value of the target type, from an FFI value of type Self::FfiType.
    ///
    /// This trait method is used for receiving data from the foreign language code in rust,
    /// by (hopefully cheaply!) converting it from a low-level FFI value of type Self::FfiType
    /// into a high-level rust value of the target type.
    ///
    /// Since we cannot statically guarantee that the foreign-language code will send valid
    /// values of type Self::FfiType, this method is fallible.
    fn try_lift(v: Self::FfiType) -> Result<Self>;

    /// Write a rust value into a buffer, to send over the FFI in serialized form.
    ///
    /// This trait method can be used for sending data from rust to the foreign language code,
    /// in cases where we're not able to use a special-purpose FFI type and must fall back to
    /// sending serialized bytes.
    fn write<B: BufMut>(&self, buf: &mut B);

    /// Read a rust value from a buffer, received over the FFI in serialized form.
    ///
    /// This trait method can be used for receiving data from the foreign language code in rust,
    /// in cases where we're not able to use a special-purpose FFI type and must fall back to
    /// receiving serialized bytes.
    ///
    /// Since we cannot statically guarantee that the foreign-language code will send valid
    /// serialized bytes for the target type, this method is fallible.
    fn try_read<B: Buf>(buf: &mut B) -> Result<Self>;
}

/// A helper function to lower a type by serializing it into a buffer.
///
/// For complex types were it's too fiddly or too unsafe to convert them into a special-purpose
/// C-compatible value, you can use this helper function to implement `lower()` in terms of `write()`
/// and pass the value as a serialized buffer of bytes.
pub fn lower_into_buffer<T: ViaFfi>(value: T) -> RustBuffer {
    let mut buf = Vec::new();
    ViaFfi::write(&value, &mut buf);
    RustBuffer::from_vec(buf)
}

/// A helper function to lift a type by deserializing it from a buffer.
///
/// For complex types were it's too fiddly or too unsafe to convert them into a special-purpose
/// C-compatible value, you can use this helper function to implement `lift()` in terms of `read()`
/// and receive the value as a serialzied byte buffer.
pub fn try_lift_from_buffer<T: ViaFfi>(buf: RustBuffer) -> Result<T> {
    let vec = buf.destroy_into_vec();
    let mut buf = vec.as_slice();
    let value = <T as ViaFfi>::try_read(&mut buf)?;
    if buf.remaining() != 0 {
        bail!("junk data left in buffer after lifting")
    }
    Ok(value)
}

/// A helper function to ensure we don't read past the end of a buffer.
///
/// Rust won't actually let us read past the end of a buffer, but the `Buf` trait does not support
/// returning an explicit error in this case, and will instead panic. This is a look-before-you-leap
/// helper function to instead return an explicit error, to help with debugging.
pub fn check_remaining<B: Buf>(buf: &B, num_bytes: usize) -> Result<()> {
    if buf.remaining() < num_bytes {
        bail!(format!(
            "not enough bytes remaining in buffer ({} < {})",
            buf.remaining(),
            num_bytes
        ));
    }
    Ok(())
}

/// Blanket implementation of ViaFfi for numeric primitives.
///
/// Numeric primitives have a straightforward mapping into C-compatible numeric types,
/// sice they are themselves a C-compatible numeric type!
macro_rules! impl_via_ffi_for_num_primitive {
    ($($T:ty,)+) => { impl_via_ffi_for_num_primitive!($($T),+); };
    ($($T:ty),*) => {
            $(
                paste! {
                    unsafe impl ViaFfi for $T {
                        type FfiType = Self;

                        fn lower(self) -> Self::FfiType {
                            self
                        }

                        fn try_lift(v: Self::FfiType) -> Result<Self> {
                            Ok(v)
                        }

                        fn write<B: BufMut>(&self, buf: &mut B) {
                            buf.[<put_ $T>](*self);
                        }

                        fn try_read<B: Buf>(buf: &mut B) -> Result<Self> {
                            check_remaining(buf, std::mem::size_of::<$T>())?;
                            Ok(buf.[<get_ $T>]())
                        }
                    }
                }
            )*
    };
}

impl_via_ffi_for_num_primitive! {
    i8, u8, i16, u16, i32, u32, i64, u64, f32, f64
}

/// Support for passing boolean values via the FFI.
///
/// Booleans are passed as an `i8` in order to avoid problems with handling
/// C-compatible boolean values on JVM-based languages.
unsafe impl ViaFfi for bool {
    type FfiType = i8;

    fn lower(self) -> Self::FfiType {
        if self {
            1
        } else {
            0
        }
    }

    fn try_lift(v: Self::FfiType) -> Result<Self> {
        Ok(match v {
            0 => false,
            1 => true,
            _ => bail!("unexpected byte for Boolean"),
        })
    }

    fn write<B: BufMut>(&self, buf: &mut B) {
        buf.put_i8(ViaFfi::lower(*self));
    }

    fn try_read<B: Buf>(buf: &mut B) -> Result<Self> {
        check_remaining(buf, 1)?;
        ViaFfi::try_lift(buf.get_i8())
    }
}

/// Support for passing Strings via the FFI.
///
/// Unlike many other implementations of `ViaFfi`, this passes a struct containing
/// a raw pointer rather than copying the data from one side to the other. This is a
/// safety hazard, but turns out to be pretty nice for useability. This struct
/// *must* be a valid `RustBuffer` and it *must* contain valid utf-8 data (in other
/// words, it *must* be a `Vec<u8>` suitable for use as an actual rust `String`).
///
/// When serialized in a buffer, strings are represented as a i32 byte length
/// followed by utf8-encoded bytes. (It's a signed integer because unsigned types are
/// currently experimental in Kotlin).
unsafe impl ViaFfi for String {
    type FfiType = RustBuffer;

    // This returns a struct with a raw pointer to the underlying bytes, so it's very
    // important that it consume ownership of the String, which is relinquished to the
    // foreign language code (and can be restored by it passing the pointer back).
    fn lower(self) -> Self::FfiType {
        RustBuffer::from_vec(self.into_bytes())
    }

    // The argument here *must* be a uniquely-owned `RustBuffer` previously obtained
    // from `lower` above, and hence must be the bytes of a valid rust string.
    fn try_lift(v: Self::FfiType) -> Result<Self> {
        let v = v.destroy_into_vec();
        // This turns the buffer back into a `String` without copying the data
        // and without re-checking it for validity of the utf8. If the `RustBuffer`
        // came from a valid String then there's no point in re-checking the utf8,
        // and if it didn't then bad things are probably going to happen regardless
        // of whether we check for valid utf8 data or not.
        Ok(unsafe { String::from_utf8_unchecked(v) })
    }

    fn write<B: BufMut>(&self, buf: &mut B) {
        // N.B. `len()` gives us the length in bytes, not in chars or graphemes.
        // TODO: it would be nice not to panic here.
        let len = i32::try_from(self.len()).unwrap();
        buf.put_i32(len); // We limit strings to u32::MAX bytes
        buf.put(self.as_bytes());
    }

    fn try_read<B: Buf>(buf: &mut B) -> Result<Self> {
        check_remaining(buf, 4)?;
        let len = usize::try_from(buf.get_i32())?;
        check_remaining(buf, len)?;
        let bytes = &buf.bytes()[..len];
        let res = String::from_utf8(bytes.to_vec())?;
        buf.advance(len);
        Ok(res)
    }
}

/// Support for passing optional values via the FFI.
///
/// Optional values are currently always passed by serializing to a buffer.
/// We write either a zero byte for `None`, or a one byte followed by the containing
/// item for `Some`.
///
/// In future we could do the same optimization as rust uses internally, where the
/// `None` option is represented as a null pointer and the `Some` as a valid pointer,
/// but that seems more fiddly and less safe in the short term, so it can wait.
unsafe impl<T: ViaFfi> ViaFfi for Option<T> {
    type FfiType = RustBuffer;

    fn lower(self) -> Self::FfiType {
        lower_into_buffer(self)
    }

    fn try_lift(v: Self::FfiType) -> Result<Self> {
        try_lift_from_buffer(v)
    }

    fn write<B: BufMut>(&self, buf: &mut B) {
        match self {
            None => buf.put_i8(0),
            Some(v) => {
                buf.put_i8(1);
                ViaFfi::write(v, buf);
            }
        }
    }

    fn try_read<B: Buf>(buf: &mut B) -> Result<Self> {
        check_remaining(buf, 1)?;
        Ok(match buf.get_i8() {
            0 => None,
            1 => Some(<T as ViaFfi>::try_read(buf)?),
            _ => bail!("unexpected tag byte for Option"),
        })
    }
}

/// Support for passing vectors of values via the FFI.
///
/// Vectors are currently always passed by serializing to a buffer.
/// We write a `i32` item count followed by each item in turn.
/// (It's a signed type due to limits of the JVM).
///
/// Ideally we would pass `Vec<u8>` directly as a `RustBuffer` rather
/// than serializing, and perhaps even pass other vector types using a
/// similar struct. But that's for future work.
unsafe impl<T: ViaFfi> ViaFfi for Vec<T> {
    type FfiType = RustBuffer;

    fn lower(self) -> Self::FfiType {
        lower_into_buffer(self)
    }

    fn try_lift(v: Self::FfiType) -> Result<Self> {
        try_lift_from_buffer(v)
    }

    fn write<B: BufMut>(&self, buf: &mut B) {
        // TODO: would be nice not to panic here :-/
        let len = i32::try_from(self.len()).unwrap();
        buf.put_i32(len); // We limit arrays to i32::MAX items
        for item in self.iter() {
            ViaFfi::write(item, buf);
        }
    }

    fn try_read<B: Buf>(buf: &mut B) -> Result<Self> {
        check_remaining(buf, 4)?;
        let len = usize::try_from(buf.get_i32())?;
        let mut vec = Vec::with_capacity(len);
        for _ in 0..len {
            vec.push(<T as ViaFfi>::try_read(buf)?)
        }
        Ok(vec)
    }
}

/// Support for associative arrays via the FFI.
/// Note that because of webidl limitations,
/// the key must always be of the String type.
///
/// HashMaps are currently always passed by serializing to a buffer.
/// We write a `i32` entries count followed by each entry (string
/// key followed by the value) in turn.
/// (It's a signed type due to limits of the JVM).
unsafe impl<V: ViaFfi> ViaFfi for HashMap<String, V> {
    type FfiType = RustBuffer;

    fn lower(self) -> Self::FfiType {
        lower_into_buffer(self)
    }

    fn try_lift(v: Self::FfiType) -> Result<Self> {
        try_lift_from_buffer(v)
    }

    fn write<B: BufMut>(&self, buf: &mut B) {
        // TODO: would be nice not to panic here :-/
        let len = i32::try_from(self.len()).unwrap();
        buf.put_i32(len); // We limit HashMaps to i32::MAX entries
        for (key, value) in self.iter() {
            ViaFfi::write(key, buf);
            ViaFfi::write(value, buf);
        }
    }

    fn try_read<B: Buf>(buf: &mut B) -> Result<Self> {
        check_remaining(buf, 4)?;
        let len = usize::try_from(buf.get_i32())?;
        let mut map = HashMap::with_capacity(len);
        for _ in 0..len {
            let key = String::try_read(buf)?;
            let value = <V as ViaFfi>::try_read(buf)?;
            map.insert(key, value);
        }
        Ok(map)
    }
}

#[cfg(feature = "serde_json")]
use serde_json::Value;

/// Support for passing JSON objects.
///
/// For correctness and speed of implementation, objects are serialized into a String.
/// This can definitely be improved in efficiency, however high-quality JSON parsers and
/// stringifiers are available on both sides of the FFI.
///
/// Only top-level objects are supported: i.e. the value must be a `Value::Object`.
/// If the Rust side tries to pass back a `JSONArray` or a scalar, and the foreign language side
/// is expecting a dictionary shaped value, it would be a failure.
///
#[cfg(feature = "serde_json")]
unsafe impl ViaFfi for Value {
    type FfiType = RustBuffer;

    fn lower(self) -> Self::FfiType {
        to_supported_string(&self)
            .map(String::lower)
            .expect("Unserializable JSON")
    }

    fn try_lift(v: Self::FfiType) -> Result<Self> {
        let json_string = String::try_lift(v)?;
        Ok(serde_json::from_str(&json_string)?)
    }

    fn write<B: BufMut>(&self, buf: &mut B) {
        to_supported_string(&self)
            .map(|s| s.write(buf))
            .expect("Unserializable JSON")
    }

    fn try_read<B: Buf>(buf: &mut B) -> Result<Self> {
        let json_string = String::try_read(buf)?;
        Ok(serde_json::from_str(&json_string)?)
    }
}

#[cfg(feature = "serde_json")]
fn to_supported_string(value: &Value) -> Result<String> {
    match value {
        Value::Object(_) => Ok(serde_json::to_string(value)?),
        _ => {
            // Currently we don't support JSONArrays or scalar values
            // at the top level. We'd like to catch this at compile time,
            // but because serde can load everything into a single enum,
            // we have to rely on runtime checking.
            panic!("Only serde_json::Value::Object values can be passed across the FFI");
        }
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn trybuild_ui_tests() {
        let t = trybuild::TestCases::new();
        t.compile_fail("tests/ui/*.rs");
    }
}
