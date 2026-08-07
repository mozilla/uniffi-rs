/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

/// Support for reading a slice of foreign-language-allocated bytes over the FFI.
///
/// Foreign language code can pass a slice of bytes by providing a data pointer
/// and length, and this struct provides a convenient wrapper for working with
/// that pair. Naturally, this can be tremendously unsafe! So here are the details:
///
///   * The foreign language code must ensure the provided buffer stays alive
///     and unchanged for the duration of the call to which the `ForeignBytes`
///     struct was provided.
///
/// To work with the bytes in Rust code, use `as_slice()` to view the data
/// as a `&[u8]`.
///
/// Implementation note: all the fields of this struct are private and it has no
/// constructors, so consuming crates cant create instances of it. If you've
/// got a `ForeignBytes`, then you received it over the FFI and are assuming that
/// the foreign language code is upholding the above invariants.
///
/// This struct is based on `ByteBuffer` from the `ffi-support` crate, but modified
/// to give a read-only view of externally-provided bytes.
#[repr(C)]
pub struct ForeignBytes {
    /// The length of the pointed-to data.
    /// We use an `i32` for compatibility with JNA.
    pub(crate) len: i32,
    /// The pointer to the foreign-owned bytes.
    pub(crate) data: *const u8,
}

impl ForeignBytes {
    /// Creates a `ForeignBytes` from its constituent fields.
    ///
    /// This is intended mainly as an internal convenience function and should not
    /// be used outside of this module.
    ///
    /// # Safety
    ///
    /// You must ensure that the raw parts uphold the documented invariants of this class.
    pub unsafe fn from_raw_parts(data: *const u8, len: i32) -> Self {
        Self { len, data }
    }

    /// View the foreign bytes as a `&[u8]`.
    ///
    /// # Panics
    ///
    /// Panics if the provided struct has a null pointer but non-zero length.
    /// Panics if the provided length is negative.
    pub fn as_slice(&self) -> &[u8] {
        if self.data.is_null() {
            assert!(self.len == 0, "null ForeignBytes had non-zero length");
            &[]
        } else {
            unsafe { std::slice::from_raw_parts(self.data, self.len()) }
        }
    }

    /// Get the length of this slice of bytes.
    ///
    /// # Panics
    ///
    /// Panics if the provided length is negative.
    pub fn len(&self) -> usize {
        self.len
            .try_into()
            .expect("bytes length negative or overflowed")
    }

    /// Returns true if the length of this slice of bytes is 0.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

impl std::borrow::Borrow<[u8]> for ForeignBytes {
    fn borrow(&self) -> &[u8] {
        self.as_slice()
    }
}

unsafe impl<UT> crate::Lift<UT> for ForeignBytes {
    type FfiType = ForeignBytes;

    fn try_lift(v: Self::FfiType) -> crate::Result<Self> {
        Ok(v)
    }

    fn try_read(_buf: &mut &[u8]) -> crate::Result<Self> {
        anyhow::bail!("ForeignBytes cannot be read from a RustBuffer")
    }
}

impl<UT> crate::TypeId<UT> for ForeignBytes {
    const TYPE_ID_META: crate::MetadataBuffer =
        crate::MetadataBuffer::from_code(crate::metadata::codes::TYPE_VEC)
            .concat(<u8 as crate::TypeId<UT>>::TYPE_ID_META);
}

/// Mutable sibling of [`ForeignBytes`] for zero-copy `&mut [u8]` / `[ByMutRef]
/// bytes` arguments.
///
/// The C ABI is identical to `ForeignBytes` (`{ i32 len, pointer data }`) —
/// pointer const-ness is not part of the ABI — so both map to the same
/// `FfiType`. The only difference is Rust-side: this exposes `as_mut_slice`
/// and `BorrowMut<[u8]>`, so Rust can write through the pointer.
///
/// Because the call is **synchronous** and the foreign side keeps the buffer
/// alive and pinned for the duration of the call, Rust's writes are
/// immediately visible to the caller with no write-back copy.
#[repr(C)]
pub struct ForeignBytesMut {
    /// The length of the pointed-to data. `i32` for JNA compatibility.
    pub(crate) len: i32,
    /// The pointer to the foreign-owned, mutable bytes.
    pub(crate) data: *mut u8,
}

impl ForeignBytesMut {
    /// Creates a `ForeignBytesMut` from its constituent fields.
    ///
    /// # Safety
    ///
    /// You must ensure that the raw parts uphold the documented invariants of
    /// this class: the buffer stays alive, pinned, and exclusively borrowed for
    /// the duration of the call.
    pub unsafe fn from_raw_parts(data: *mut u8, len: i32) -> Self {
        Self { len, data }
    }

    /// View the foreign bytes as a `&[u8]`.
    ///
    /// # Panics
    ///
    /// Panics if the struct has a null pointer but non-zero length, or if the
    /// length is negative.
    pub fn as_slice(&self) -> &[u8] {
        if self.data.is_null() {
            assert!(self.len == 0, "null ForeignBytesMut had non-zero length");
            &[]
        } else {
            unsafe { std::slice::from_raw_parts(self.data, self.len()) }
        }
    }

    /// View the foreign bytes as a `&mut [u8]`. Writes land directly in the
    /// foreign-owned backing store.
    ///
    /// # Panics
    ///
    /// Panics if the struct has a null pointer but non-zero length, or if the
    /// length is negative.
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        if self.data.is_null() {
            assert!(self.len == 0, "null ForeignBytesMut had non-zero length");
            &mut []
        } else {
            unsafe { std::slice::from_raw_parts_mut(self.data, self.len()) }
        }
    }

    /// Get the length of this slice of bytes.
    ///
    /// # Panics
    ///
    /// Panics if the length is negative.
    pub fn len(&self) -> usize {
        self.len
            .try_into()
            .expect("bytes length negative or overflowed")
    }

    /// Returns true if the length of this slice of bytes is 0.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

impl std::borrow::Borrow<[u8]> for ForeignBytesMut {
    fn borrow(&self) -> &[u8] {
        self.as_slice()
    }
}

impl std::borrow::BorrowMut<[u8]> for ForeignBytesMut {
    fn borrow_mut(&mut self) -> &mut [u8] {
        self.as_mut_slice()
    }
}

unsafe impl<UT> crate::Lift<UT> for ForeignBytesMut {
    type FfiType = ForeignBytesMut;

    fn try_lift(v: Self::FfiType) -> crate::Result<Self> {
        Ok(v)
    }

    fn try_read(_buf: &mut &[u8]) -> crate::Result<Self> {
        anyhow::bail!("ForeignBytesMut cannot be read from a RustBuffer")
    }
}

impl<UT> crate::TypeId<UT> for ForeignBytesMut {
    const TYPE_ID_META: crate::MetadataBuffer =
        crate::MetadataBuffer::from_code(crate::metadata::codes::TYPE_VEC)
            .concat(<u8 as crate::TypeId<UT>>::TYPE_ID_META);
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_foreignbytes_access() {
        let v = [1u8, 2, 3];
        let fbuf = unsafe { ForeignBytes::from_raw_parts(v.as_ptr(), 3) };
        assert_eq!(fbuf.len(), 3);
        assert_eq!(fbuf.as_slice(), &[1u8, 2, 3]);
    }

    #[test]
    fn test_foreignbytes_empty() {
        let v = Vec::<u8>::new();
        let fbuf = unsafe { ForeignBytes::from_raw_parts(v.as_ptr(), 0) };
        assert_eq!(fbuf.len(), 0);
        assert_eq!(fbuf.as_slice(), &[0u8; 0]);
    }

    #[test]
    fn test_foreignbytes_null_means_empty() {
        let fbuf = unsafe { ForeignBytes::from_raw_parts(std::ptr::null_mut(), 0) };
        assert_eq!(fbuf.as_slice(), &[0u8; 0]);
    }

    #[test]
    #[should_panic]
    fn test_foreignbytes_null_must_have_zero_length() {
        let fbuf = unsafe { ForeignBytes::from_raw_parts(std::ptr::null_mut(), 12) };
        fbuf.as_slice();
    }

    #[test]
    #[should_panic]
    fn test_foreignbytes_provided_len_must_be_non_negative() {
        let v = [0u8, 1, 2];
        let fbuf = unsafe { ForeignBytes::from_raw_parts(v.as_ptr(), -1) };
        fbuf.as_slice();
    }

    #[test]
    fn test_foreignbytes_borrow_as_slice() {
        use std::borrow::Borrow;
        let v = [10u8, 20, 30];
        let fbuf = unsafe { ForeignBytes::from_raw_parts(v.as_ptr(), 3) };
        let borrowed: &[u8] = Borrow::<[u8]>::borrow(&fbuf);
        assert_eq!(borrowed, &[10u8, 20, 30]);
    }

    #[test]
    fn test_foreignbytes_lift() {
        use crate::{Lift, UniFfiTag};
        let v = [1u8, 2, 3];
        let fbuf = unsafe { ForeignBytes::from_raw_parts(v.as_ptr(), 3) };
        let lifted: ForeignBytes = <ForeignBytes as Lift<UniFfiTag>>::try_lift(fbuf).unwrap();
        assert_eq!(lifted.as_slice(), &[1u8, 2, 3]);
    }

    #[test]
    fn test_foreignbytesmut_access() {
        let mut v = [1u8, 2, 3];
        let mut fbuf = unsafe { ForeignBytesMut::from_raw_parts(v.as_mut_ptr(), 3) };
        assert_eq!(fbuf.len(), 3);
        assert!(!fbuf.is_empty());
        assert_eq!(fbuf.as_slice(), &[1u8, 2, 3]);
        assert_eq!(fbuf.as_mut_slice(), &mut [1u8, 2, 3]);
    }

    #[test]
    fn test_foreignbytesmut_empty() {
        let mut v = Vec::<u8>::new();
        let fbuf = unsafe { ForeignBytesMut::from_raw_parts(v.as_mut_ptr(), 0) };
        assert_eq!(fbuf.len(), 0);
        assert!(fbuf.is_empty());
        assert_eq!(fbuf.as_slice(), &[0u8; 0]);
    }

    #[test]
    fn test_foreignbytesmut_null_means_empty() {
        let mut fbuf = unsafe { ForeignBytesMut::from_raw_parts(std::ptr::null_mut(), 0) };
        assert_eq!(fbuf.as_slice(), &[0u8; 0]);
        assert_eq!(fbuf.as_mut_slice(), &mut [0u8; 0]);
    }

    #[test]
    #[should_panic]
    fn test_foreignbytesmut_null_must_have_zero_length() {
        let mut fbuf = unsafe { ForeignBytesMut::from_raw_parts(std::ptr::null_mut(), 12) };
        fbuf.as_mut_slice();
    }

    #[test]
    #[should_panic]
    fn test_foreignbytesmut_provided_len_must_be_non_negative() {
        let mut v = [0u8, 1, 2];
        let fbuf = unsafe { ForeignBytesMut::from_raw_parts(v.as_mut_ptr(), -1) };
        fbuf.as_slice();
    }

    #[test]
    fn test_foreignbytesmut_borrow_mut_and_mutate() {
        use std::borrow::BorrowMut;
        let mut v = [10u8, 20, 30];
        let mut fbuf = unsafe { ForeignBytesMut::from_raw_parts(v.as_mut_ptr(), 3) };
        let borrowed: &mut [u8] = BorrowMut::<[u8]>::borrow_mut(&mut fbuf);
        borrowed[0] = 99;
        // Writes through the mutable slice are visible in the backing store.
        assert_eq!(v, [99u8, 20, 30]);
    }

    #[test]
    fn test_foreignbytesmut_lift() {
        use crate::{Lift, UniFfiTag};
        let mut v = [1u8, 2, 3];
        let fbuf = unsafe { ForeignBytesMut::from_raw_parts(v.as_mut_ptr(), 3) };
        let mut lifted: ForeignBytesMut =
            <ForeignBytesMut as Lift<UniFfiTag>>::try_lift(fbuf).unwrap();
        assert_eq!(lifted.as_mut_slice(), &mut [1u8, 2, 3]);
    }
}
