// Everybody gets basic bytebuffer allocation and freeing, since it's needed
// for passing complex types over the FFI.
#[no_mangle]
pub extern "C" fn {{ ci.ffi_bytebuffer_alloc().name() }}(size: u32) -> ffi_support::ByteBuffer {
    ffi_support::ByteBuffer::new_with_size(size.max(0) as usize)
}

ffi_support::define_bytebuffer_destructor!({{ ci.ffi_bytebuffer_free().name() }});