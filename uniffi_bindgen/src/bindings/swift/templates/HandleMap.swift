// Generate map IDs that are likely to be unique
fileprivate var uniffiMapIdCounter: UInt64 = {{ ci.namespace_hash() }} & 0xFFFF
fileprivate func uniffiNextMapId() -> UInt64 {
    let mapId = uniffiMapIdCounter
    uniffiMapIdCounter = (uniffiMapIdCounter + 1) & 0xFFFF
    return mapId
}

// Manage handles for objects that are passed across the FFI
//
// See the `uniffi_core::HandleAlloc` trait for the semantics of each method
fileprivate class UniffiHandleMap<T> {

    // Map ID, shifted into the top 16 bits
    private let mapId: UInt64 = uniffiNextMapId() << 48
    private let lock: NSLock = NSLock()
    private var map: [UInt64: T] = [:]
    // Note: foreign handles are always odd
    private var keyCounter: UInt64 = 1

    private func nextKey() -> UInt64 {
        let key = keyCounter
        keyCounter = (keyCounter + 2) & 0xFFFF_FFFF_FFFF
        return key
    }

    private func makeHandle(_ key: UInt64) -> UInt64 {
        return key | mapId
    }

    private func key(_ handle: UInt64) -> UInt64 {
        if (handle & 0xFFFF_0000_0000_0000 != mapId) {
            fatalError("Handle map ID mismatch")
        }
        return handle & 0xFFFF_FFFF_FFFF
    }

    func newHandle(obj: T) -> UInt64 {
        lock.withLock {
            let key = nextKey()
            map[key] = obj
            return makeHandle(key)
        }
    }
 
    func get(handle: UInt64) -> T {
        lock.withLock {
            guard let obj = map[key(handle)] else {
                fatalError("handlemap key error: was the handle used after being freed?")
            }
            return obj
        }
    }

    func cloneHandle(handle: UInt64) -> UInt64 {
        lock.withLock {
            guard let obj = map[key(handle)] else {
                fatalError("handlemap key error: was the handle used after being freed?")
            }
            let key = nextKey()
            map[key] = obj
            return makeHandle(key)
        }
    }
 
    @discardableResult
    func consumeHandle(handle: UInt64) -> T {
        lock.withLock {
            guard let obj = map.removeValue(forKey: key(handle)) else {
                fatalError("handlemap key error: was the handle used after being freed?")
            }
            return obj
        }
    }
}
