/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use anyhow::{Result, bail};
use bytes::buf::{Buf, BufMut};
use ffi_support::ByteBuffer;

use super::ViaFfi;

/// For returning records (and related rich data types like sequences) over the FFI
/// we serialize them into a ByteBuffer. This is a little bit of serialization machinery
/// to help make that work.
///
/// The `Serializer` struct is a little wrapper around a vector of bytes, whih can be used
/// to recursively serialize a type and its components into a `ByteBuffer`.

pub struct Serializer {
  buffer: Vec<u8>,
}

impl Serializer {
  pub fn new() -> Self {
    Serializer {
      buffer: Vec::new(),
    }
  }

  pub fn serialize<T: Serializable>(&mut self, value: &T) {
    value.serialize_into(self);
  }

  pub fn finalize(self) -> Vec<u8> {
    self.buffer
  }
}

/// Any type that wants to pass over the FFI as a ByteBuffer should implement the
/// `Serializeable` trait. A typical implementation of `Serializeable::serialize_into`
/// would serialize each subfield in turn. (Like `serde` does I guess...it might be
/// possible to use `serde` for this, except that the deserialization happens in generated
/// code in a different language, so it's convenient to have direct control over the details).

pub trait Serializable : Sized {
  fn serialize_into(&self, buf: &mut Serializer);
}

impl<T: Serializable> Serializable for &T {
  fn serialize_into(&self, buf: &mut Serializer) {
    buf.serialize(*self);
  }
}

impl Serializable for u32 {
  fn serialize_into(&self, buf: &mut Serializer) {
    buf.buffer.put_u32(*self);
  }
}

impl Serializable for f64 {
  fn serialize_into(&self, buf: &mut Serializer) {
    buf.buffer.put_f64(*self);
  }
}

impl<T: Serializable> Serializable for Option<T> {
  fn serialize_into(&self, buf: &mut Serializer) {
    match self {
      None => buf.buffer.put_u8(0),
      Some(v) => {
        buf.buffer.put_u8(1);
        v.serialize_into(buf);
      },
    }
  }
}

/// For receiving records (and related rich data types like sequences) over the FFI
/// we deserialize them from a ByteBuffer. This is a little bit of deserialization machinery
/// to help make that work.
///
/// The `Deserializer` struct is a little wrapper around a vector of bytes, whih can be used
/// to recursively deserialize a type and its components from a `ByteBuffer`.

pub struct Deserializer<'a> {
  buffer: &'a[u8],
}

impl<'a> Deserializer<'a> {
  pub fn new(buf: &'a[u8]) -> Self {
    Deserializer {
      buffer: buf
    }
  }

  pub fn deserialize_fully<T: Deserializable>(&mut self) -> Result<T> {
    let retval = self.deserialize();
    if self.buffer.remaining() != 0 {
      panic!("junk data left in buffer after deserializing")
    }
    retval
  }

  pub fn deserialize<T: Deserializable>(&mut self) -> Result<T> {
    T::deserialize_from(self)
  }

  fn check_remaining(&self, num_bytes: usize) -> Result<()> {
    if self.buffer.remaining() < num_bytes {
      bail!("not enough bytes remaining in buffer");
    }
    Ok(())
  }
}

/// Any type that wants to be received over the FFI as a ByteBuffer should implement the
/// `Deserializeable` trait. A typical implementation of `Deserializeable::deserialize_from`
/// would deserialize each subfield in turn, basically the dual of `Serializeable` above.

pub trait Deserializable: Sized + std::fmt::Debug {
  fn deserialize_from(buf: &mut Deserializer) -> Result<Self>;
}

impl Deserializable for u32 {
  fn deserialize_from(buf: &mut Deserializer) -> Result<Self> {
    buf.check_remaining(4)?;
    let v = buf.buffer.get_u32();
    Ok(v)
  }
}

impl Deserializable for f64 {
  fn deserialize_from(buf: &mut Deserializer) -> Result<Self> {
    buf.check_remaining(8)?;
    let v = buf.buffer.get_f64();
    Ok(v)
  }
}

impl<T: Deserializable> Deserializable for Option<T> {
  fn deserialize_from(buf: &mut Deserializer) -> Result<Self> {
    buf.check_remaining(1)?;
    Ok(match buf.buffer.get_u8() {
      0 => None,
      1 => Some(T::deserialize_from(buf)?),
      _ => bail!("unexpected tag byte for Option")
    })
  }
}


// Top-level types that pass via ByteBuffer should implement `Record`, in
// order to get a free implementation of `ViaFfi`.

pub trait Record: Serializable + Deserializable {
  fn try_from_bytebuffer(buf: ByteBuffer) -> Result<Self> {
     let bytes = buf.into_vec();
     let mut buf = Deserializer::new(bytes.as_slice());
     buf.deserialize_fully()
  }

  fn into_bytebuffer(&self) -> ByteBuffer {
      let mut buf = Serializer::new();
      buf.serialize(&self);
      ByteBuffer::from_vec(buf.finalize())
  }
}

unsafe impl<T: Record> ViaFfi for T {
  type Value = ByteBuffer;
  fn into_ffi_value(self) -> Self::Value {
    self.into_bytebuffer()
  }
  fn try_from_ffi_value(v: Self::Value) -> Result<Self> {
    Self::try_from_bytebuffer(v)
  }
}

// Some default types pass over the FFI as records.

impl<T: Serializable + Deserializable> Record for Option<T> {}