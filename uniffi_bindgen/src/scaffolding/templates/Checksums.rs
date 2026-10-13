{%- for (name, checksum) in ci.iter_checksums() %}
#[unsafe(no_mangle)]
#[doc(hidden)]
pub extern "C" fn r#{{ name }}() -> u16 {
    {{ checksum }}
}

{%- if pointer_ffi %}
#[unsafe(no_mangle)]
#[doc(hidden)]
pub unsafe extern "C" fn r#{{ name|pointer_ffi_symbol_name }}(uniffi_ffi_buffer: *mut u8) {
    // Safety: we follow the pointer FFI when reading/writing from the buffer
    unsafe {
        let mut uniffi_return_buf = ::std::slice::from_raw_parts_mut(
            uniffi_ffi_buffer,
            ::uniffi::ffi_buffer_size!((u16)),
        );
        <u16 as ::uniffi::FfiSerialize>::write(&mut uniffi_return_buf, {{ checksum }});
    }
}
{%- endif %}
{%- endfor %}
