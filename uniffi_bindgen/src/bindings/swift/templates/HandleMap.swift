// Initial value and increment amount for handles. 
// These ensure that SWIFT handles always have the lowest bit set
fileprivate let UNIFFI_HANDLEMAP_INITIAL: UInt64 = 1
fileprivate let UNIFFI_HANDLEMAP_DELTA: UInt64 = 2

fileprivate final class UniffiHandleMap<T>: @unchecked Sendable {
    // All mutation happens with this lock held, which is why we implement @unchecked Sendable.
    private let lock = NSLock()
    private var map: [UInt64: T] = [:]
    private var currentHandle: UInt64 = UNIFFI_HANDLEMAP_INITIAL

    func insert(obj: T) -> UInt64 {
        lock.withLock {
            return doInsert(obj)
        }
    }

    // Low-level insert function, this assumes `lock` is held.
    private func doInsert(_ obj: T) -> UInt64 {
        let handle = currentHandle
        currentHandle += UNIFFI_HANDLEMAP_DELTA
        map[handle] = obj
        return handle
    }

     func get(handle: UInt64) throws -> T {
        try lock.withLock {
            guard let obj = map[handle] else {
                throw UniffiInternalError.unexpectedStaleHandle
            }
            return obj
        }
    }

     func clone(handle: UInt64) throws -> UInt64 {
        try lock.withLock {
            guard let obj = map[handle] else {
                throw UniffiInternalError.unexpectedStaleHandle
            }
            return doInsert(obj)
        }
    }

    @discardableResult
    func remove(handle: UInt64) throws -> T {
        try lock.withLock {
            guard let obj = map.removeValue(forKey: handle) else {
                throw UniffiInternalError.unexpectedStaleHandle
            }
            return obj
        }
    }

    var count: Int {
        get {
            map.count
        }
    }
}

