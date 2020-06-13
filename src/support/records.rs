/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use anyhow::{Result, bail};
use bytes::buf::{Buf, BufMut};


pub struct Serializer {
  buffer: Vec<u8>,
}

impl Serializer {
  pub fn new() -> Self {
    Serializer {
      buffer: Vec::new(),
    }
  }

  pub fn serialize<T: Serializable>(&mut self, value: T) {
    value.serialize_into(self);
  }

  pub fn finalize(self) -> Vec<u8> {
    self.buffer
  }
}

pub trait Serializable {
  fn serialize_into(&self, buf: &mut Serializer);
}


impl Serializable for &u32 {
  fn serialize_into(&self, buf: &mut Serializer) {
    buf.buffer.put_u32(**self);
  }
}


pub struct Deserializer<'a> {
  buffer: &'a[u8],
}

impl<'a> Deserializer<'a> {
  pub fn new(buf: &'a[u8]) -> Self {
    println!("Deserializer from buf of {:?} bytes {:?}", buf.len(), buf);
    Deserializer {
      buffer: buf
    }
  }

  pub fn deserialize_fully<T: Deserializable>(&mut self) -> Result<T> {
    let retval = self.deserialize();
    if self.buffer.remaining() != 0 {
      panic!("junk data left in buffer after deserializing")
    }
    println!("Deserialized record {:?}", retval);
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

pub trait Deserializable: Sized + std::fmt::Debug {
  fn deserialize_from(buf: &mut Deserializer) -> Result<Self>;
}

impl Deserializable for u32 {
  fn deserialize_from(buf: &mut Deserializer) -> Result<Self> {
    buf.check_remaining(4)?;
    let v = buf.buffer.get_u32();
    println!("DESER {}", v);
    Ok(v)
  }
}