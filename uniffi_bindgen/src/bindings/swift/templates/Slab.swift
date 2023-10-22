func uniffiHandleIsFromRust(_ handle: Int64) -> Bool {
    return (handle & 0x0001_0000_0000) == 0
}

fileprivate class UniffiSlab<T> {
    private let slabHandle = {{ ci.ffi_slab_new().name() }}()
    // TODO: use a read-write lock.
    //
    // Swift articles suggest using the `pthreads` version.  Can we just switch to that?
    // If not, maybe we could define some Rust FFI functions to handle it, although it's hard with
    // the `std::sync::RwLock` since that uses a guard API
    private let lock = NSLock()
    var items = Array<T?>()

    private func index(_ handle: Int64) -> Int {
        return Int(handle & 0xFFFF)
    }

    internal func insert(value: T) throws -> Int64 {
        let handle = {{ ci.ffi_slab_insert().name() }}(slabHandle)
        if (handle < 0) {
            throw UniffiInternalError.slabError
        }
        let index = index(handle)
        return lock.withLock {
            while (items.count <= index) {
                items.append(nil)
            }
            items[index] = value
            return handle
        }
    }

    internal func get(handle: Int64) throws -> T {
        let result = {{ ci.ffi_slab_check_handle().name() }}(slabHandle, handle)
        if (result < 0) {
            throw UniffiInternalError.slabError
        }
        return lock.withLock {
            return items[index(handle)]!
        }
    }

    internal func incRef(handle: Int64) throws {
        let result = {{ ci.ffi_slab_inc_ref().name() }}(slabHandle, handle)
        if (result < 0) {
            throw UniffiInternalError.slabError
        }
    }

    internal func remove(handle: Int64) throws -> T {
        let result = {{ ci.ffi_slab_dec_ref().name() }}(slabHandle, handle)
        if (result < 0) {
            throw UniffiInternalError.slabError
        }
        let index = index(handle)
        return lock.withLock {
            let value = items[index]!
            if (result == 1) {
                items[index] = nil
            }
            return value
        }
    }
}
