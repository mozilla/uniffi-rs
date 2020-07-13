extension RustBuffer {
    init(bytes: [UInt8]) {
        // TODO: This also copies the buffer. Can we write directly into
        // a Rust buffer?
        let rustBuffer = {{ ci.ffi_bytebuffer_alloc().name() }}(UInt32(bytes.count))
        let pointer = UnsafeMutableBufferPointer(start: rustBuffer.data, count: Int(rustBuffer.len))
        bytes.copyBytes(to: pointer)
        self.init(len: Int64(pointer.count), data: pointer.baseAddress!)
    }

    // Frees the buffer in place. The buffer must not be used after this is
    // called.
    func deallocate() {
        {{ ci.ffi_bytebuffer_free().name() }}(self)
    }
}