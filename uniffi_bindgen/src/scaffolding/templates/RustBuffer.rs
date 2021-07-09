// Everybody gets basic buffer support, since it's needed for passing complex types over the FFI.

/// This helper allocates a new byte buffer owned by the Rust code, and returns it
/// to the foreign-language code as a `RustBuffer` struct. Callers must eventually
/// free the resulting buffer, either by explicitly calling the destructor defined below,
/// or by passing ownership of the buffer back into Rust code.
#[doc(hidden)]
#[no_mangle]
pub extern "C" fn {{ ci.ffi_rustbuffer_alloc().name() }}(size: i32, call_status: &mut uniffi::RustCallStatus) -> uniffi::RustBuffer {
    uniffi::call_with_output(call_status, || {
        uniffi::RustBuffer::new_with_size(size.max(0) as usize)
    })
}

/// This helper copies bytes owned by the foreign-language code into a new byte buffer owned
/// by the Rust code, and returns it as a `RustBuffer` struct. Callers must eventually
/// free the resulting buffer, either by explicitly calling the destructor defined below,
/// or by passing ownership of the buffer back into Rust code.
///
/// # Safety
/// This function will dereference a provided pointer in order to copy bytes from it, so
/// make sure the `ForeignBytes` struct contains a valid pointer and length.
#[doc(hidden)]
#[no_mangle]
pub unsafe extern "C" fn {{ ci.ffi_rustbuffer_from_bytes().name() }}(bytes: uniffi::ForeignBytes, call_status: &mut uniffi::RustCallStatus) -> uniffi::RustBuffer {
    uniffi::call_with_output(call_status, || {
        let bytes = bytes.as_slice();
        uniffi::RustBuffer::from_vec(bytes.to_vec())
    })
}

/// Free a byte buffer that had previously been passed to the foreign language code.
///
/// # Safety
/// The argument *must* be a uniquely-owned `RustBuffer` previously obtained from a call
/// into the Rust code that returned a buffer, or you'll risk freeing unowned memory or
/// corrupting the allocator state.
#[doc(hidden)]
#[no_mangle]
pub unsafe extern "C" fn {{ ci.ffi_rustbuffer_free().name() }}(buf: uniffi::RustBuffer, call_status: &mut uniffi::RustCallStatus) {
    uniffi::call_with_output(call_status, || {
        uniffi::RustBuffer::destroy(buf)
    })
}

/// Reserve additional capacity in a byte buffer that had previously been passed to the
/// foreign language code.
///
/// The first argument *must* be a uniquely-owned `RustBuffer` previously
/// obtained from a call into the Rust code that returned a buffer. Its underlying data pointer
/// will be reallocated if necessary and returned in a new `RustBuffer` struct.
///
/// The second argument must be the minimum number of *additional* bytes to reserve
/// capacity for in the buffer; it is likely to reserve additional capacity in practice
/// due to amortized growth strategy of Rust vectors.
///
/// # Safety
/// The first argument *must* be a uniquely-owned `RustBuffer` previously obtained from a call
/// into the Rust code that returned a buffer, or you'll risk freeing unowned memory or
/// corrupting the allocator state.
#[doc(hidden)]
#[no_mangle]
pub unsafe extern "C" fn {{ ci.ffi_rustbuffer_reserve().name() }}(buf: uniffi::RustBuffer, additional: i32, call_status: &mut uniffi::RustCallStatus) -> uniffi::RustBuffer {
    uniffi::call_with_output(call_status, || {
        use std::convert::TryInto;
        let additional: usize = additional.try_into().expect("additional buffer length negative or overflowed");
        let mut v = buf.destroy_into_vec();
        v.reserve(additional);
        uniffi::RustBuffer::from_vec(v)
    })
}
