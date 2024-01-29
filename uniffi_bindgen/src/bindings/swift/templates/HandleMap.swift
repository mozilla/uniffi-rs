fileprivate typealias UniffiHandle = UInt64
fileprivate class UniffiHandleMap<T> {
    private var map: [UniffiHandle: T] = [:]
    private let lock = NSLock()
    private var currentHandle: UniffiHandle = 1

    func insert(obj: T) -> UniffiHandle {
        lock.withLock {
            let handle = currentHandle
            currentHandle += 1
            map[handle] = obj
            return handle
        }
    }

     func get(handle: UniffiHandle) throws -> T {
        try lock.withLock {
            guard let obj = map[handle] else {
                throw UniffiInternalError.unexpectedStaleHandle
            }
            return obj
        }
    }

    @discardableResult
    func remove(handle: UniffiHandle) throws -> T {
        try lock.withLock {
            guard let obj = map.removeValue(forKey: handle) else {
                throw UniffiInternalError.unexpectedStaleHandle
            }
            return obj
        }
    }
}

