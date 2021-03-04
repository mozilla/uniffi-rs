fileprivate extension RustBuffer {
    // Allocate a new buffer, copying the contents of a `UInt8` array.
    init(bytes: [UInt8]) {
        let rbuf = bytes.withUnsafeBufferPointer { ptr in
            try! rustCall(UniffiInternalError.unknown("RustBuffer.init")) { err in
                {{ ci.ffi_rustbuffer_from_bytes().name() }}(ForeignBytes(bufferPointer: ptr), err)
            }
        }
        // Ref https://github.com/mozilla/uniffi-rs/issues/334 for the extra "padding" arg.
        self.init(capacity: rbuf.capacity, len: rbuf.len, data: rbuf.data, padding: 0)
    }

    // Frees the buffer in place.
    // The buffer must not be used after this is called.
    func deallocate() {
        try! rustCall(UniffiInternalError.unknown("RustBuffer.deallocate")) { err in
            {{ ci.ffi_rustbuffer_free().name() }}(self, err)
        }
    }
}

fileprivate extension ForeignBytes {
    init(bufferPointer: UnsafeBufferPointer<UInt8>) {
        // Ref https://github.com/mozilla/uniffi-rs/issues/334 for the extra "padding" args.
        self.init(len: Int32(bufferPointer.count), data: bufferPointer.baseAddress, padding: 0, padding2: 0)
    }
}
