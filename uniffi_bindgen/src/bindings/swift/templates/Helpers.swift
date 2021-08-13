

{% if ci.iter_callback_interface_definitions().len() > 0 %}

class Lock {
    init(){}
    func withLock<T>(f: () throws -> T ) rethrows -> T {
        objc_sync_enter(self)
        defer { objc_sync_exit(self) }
        return try f()
    }
}

typealias Handle = UInt64
class ConcurrentHandleMap<T: AnyObject> {
    private var leftMap: [Handle: T] = [:]
    private var rightMap: [ObjectIdentifier: Handle] = [:]

    private let lock = Lock()
    private var currentHandle: Handle = 0
    private let stride: Handle = 1

    func insert(obj: T) -> Handle {
        lock.withLock {
            rightMap[ObjectIdentifier(obj)] ?? {
                currentHandle += stride
                let handle = currentHandle
                leftMap[handle] = obj
                rightMap[ObjectIdentifier(obj)] = handle
                return handle
            }()
        }
    }

    func callWithResult<R>(handle: Handle, fn: (T) -> R) -> R {
        guard let obj = lock.withLock(f: { leftMap[handle] })
        else { fatalError("Panic: handle not in handlemap") }
        return fn(obj)
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
            let obj = leftMap.removeValue(forKey: handle)
             if let obj = obj {
                 rightMap.removeValue(forKey:  ObjectIdentifier(obj))
             }
            return obj
        }
    }
}

// Magic number for the Rust proxy to call using the same mechanism as every other method,
// to free the callback once it's dropped by Rust.
private let IDX_CALLBACK_FREE: Int32 = 0

class CallbackInternals<CallbackInterface: AnyObject> {

    let foreignCallback: ForeignCallback

    init(foreignCallback: @escaping ForeignCallback) {
        self.foreignCallback = foreignCallback
    }

    let handleMap = ConcurrentHandleMap<CallbackInterface>()

    func drop(handle: Handle) -> RustBuffer {
        handleMap.remove(handle: handle)
        return RustBuffer()
    }

    func lift(n handle: Handle) -> CallbackInterface? {
        handleMap.get(handle: handle)
    }

    fileprivate func read(from buf: Reader) throws -> CallbackInterface? {
        let handle: UInt64 = try buf.readInt()
        return lift(n: handle)
    }

    func lower(_ v: CallbackInterface) -> Handle {
        let handle = handleMap.insert(obj: v)
        return handle
        // assert(handleMap.get(handle: obj) == v, "Handle map is not returning the object we just placed there. This is a bug in the HandleMap.")
    }

    fileprivate func write(v: CallbackInterface, into buf: Writer) {
        buf.writeInt(lower(v))
    }
}

{% endif %}