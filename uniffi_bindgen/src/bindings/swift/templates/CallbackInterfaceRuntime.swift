fileprivate extension NSLock {
    func withLock<T>(f: () throws -> T) rethrows -> T {
        self.lock()
        defer { self.unlock() }
        return try f()
    }
}

fileprivate typealias Handle = UInt64
fileprivate class ConcurrentHandleMap<T> {
    private var leftMap: [Handle: T] = [:]
    private var counter: [Handle: UInt64] = [:]
    private var rightMap: [ObjectIdentifier: Handle] = [:]

    private let lock = NSLock()
    private var currentHandle: Handle = 0
    private let stride: Handle = 1

    func insert(obj: T) -> Handle {
        lock.withLock {
            let id = ObjectIdentifier(obj as AnyObject)
            let handle = rightMap[id] ?? {
                currentHandle += stride
                let handle = currentHandle
                leftMap[handle] = obj
                rightMap[id] = handle
                return handle
            }()
            counter[handle] = (counter[handle] ?? 0) + 1
            return handle
        }
    }

    func get(handle: Handle) -> T? {
        lock.withLock {
            leftMap[handle]
        }
    }

    func delete(handle: Handle) {
        remove(handle: handle)
    }

    @discardableResult
    func remove(handle: Handle) -> T? {
        lock.withLock {
            defer { counter[handle] = (counter[handle] ?? 1) - 1 }
            guard counter[handle] == 1 else { return leftMap[handle] }
            let obj = leftMap.removeValue(forKey: handle)
            if let obj = obj {
                rightMap.removeValue(forKey: ObjectIdentifier(obj as AnyObject))
            }
            return obj
        }
    }
}

// Magic number for the Rust proxy to call using the same mechanism as every other method,
// to free the callback once it's dropped by Rust.
private let IDX_CALLBACK_FREE: Int32 = 0

fileprivate class FfiConverterCallbackInterface<CallbackInterface> {
    fileprivate let handleMap = ConcurrentHandleMap<CallbackInterface>()

    func drop(handle: Handle) {
        handleMap.remove(handle: handle)
    }

    func lift(_ handle: Handle) throws -> CallbackInterface {
        guard let callback = handleMap.get(handle: handle) else {
            throw UniffiInternalError.unexpectedStaleHandle
        }
        return callback
    }

    func read(from buf: Reader) throws -> CallbackInterface {
        let handle: Handle = try buf.readInt()
        return try lift(handle)
    }

    func lower(_ v: CallbackInterface) -> Handle {
        let handle = handleMap.insert(obj: v)
        return handle
        // assert(handleMap.get(handle: obj) == v, "Handle map is not returning the object we just placed there. This is a bug in the HandleMap.")
    }

    func write(_ v: CallbackInterface, into buf: Writer) {
        buf.writeInt(lower(v))
    }
}
