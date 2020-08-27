// Everybody gets basic string support, since they're such a common data type.

/// # Safety
/// This helper receives a borrowed foreign language string and
/// copies it into an owned rust string. It is naturally tremendously unsafe,
/// because pointers. The main things to remember as a consumer are:
///
///   * You must keep the argument buffer alive for the duration of the call,
///     and avoid mutating it e.g. from other threads.
///   * You must eventually free the return value by passing it to the string
///     destructor defined below. Not just "a string destructor like the one
///     defined below", the very one defined below for this component!
///
/// This function will report a panic via its `ExternError` out arg if given
/// a null or invalid pointer, or a buffer containing non-utf8 bytes.
#[no_mangle]
pub unsafe extern "C" fn {{ ci.ffi_string_alloc_from().name() }}(cstr: *const std::os::raw::c_char, err: &mut uniffi::deps::ffi_support::ExternError) -> *mut std::os::raw::c_char {
    uniffi::deps::ffi_support::call_with_output(err, || {
        // This logic was copied from ffi_support::FfiStr, changed to panic on invalid data.
        // We should figure out whether/how to move this back into ffi_support.
        if cstr.is_null() {
            panic!("null pointer provided to string_alloc_from");
        }
        // This copies the data into a rust-allocated string.
        let cstr = std::ffi::CStr::from_ptr(cstr);
        let s = cstr.to_str().expect("Invalid utf8 in foreign string buffer");
        // And this lowers it back into a rust-owned pointer to return over the FFI.
        <String as uniffi::ViaFfi>::lower(s.to_string())
    })
}

/// Free a String that had previously been passed to the foreign language code.
///
/// # Safety
/// In order to free the string, Rust takes ownership of a raw pointer
/// which is an unsafe operation.
/// The argument *must* be a uniquely-owned pointer previously obtained from a call
/// into the rust code that returned a string.
#[no_mangle]
pub unsafe extern "C" fn {{ ci.ffi_string_free().name() }}(cstr: *mut std::os::raw::c_char) {
    // We deliberately don't check the `Result` here, so that callers don't need to pass an out `err`.
    // There was nothing for us to free in the failure case anyway, so no point in propagating the error.
    let s = <String as uniffi::ViaFfi>::try_lift(cstr);
    drop(s)
}
