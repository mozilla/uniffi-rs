// An error type for FFI errors. These errors occur at the UniFFI level, not
// the library level.
fileprivate enum UniffiInternalError: LocalizedError {
    case bufferOverflow
    case incompleteData
    case unexpectedOptionalTag
    case unexpectedEnumCase
    case unexpectedNullPointer
    case unexpectedRustCallStatusCode
    case unexpectedRustCallError
    case rustPanic(_ message: String)

    public var errorDescription: String? {
        switch self {
        case .bufferOverflow: return "Reading the requested value would read past the end of the buffer"
        case .incompleteData: return "The buffer still has data after lifting its containing value"
        case .unexpectedOptionalTag: return "Unexpected optional tag; should be 0 or 1"
        case .unexpectedEnumCase: return "Raw enum value doesn't match any cases"
        case .unexpectedNullPointer: return "Raw pointer value was null"
        case .unexpectedRustCallStatusCode: return "Unexpected RustCallStatus code"
        case .unexpectedRustCallError: return "CALL_ERROR but no errorClass specified"
        case let .rustPanic(message): return message
        }
    }
}

fileprivate let CALL_SUCCESS: Int8 = 0
fileprivate let CALL_ERROR: Int8 = 1
fileprivate let CALL_PANIC: Int8 = 2

fileprivate extension RustCallStatus {
    init() {
        self.init(
            code: CALL_SUCCESS,
            errorBuf: RustBuffer.init(
                capacity: 0,
                len: 0,
                data: nil
            )
        )
    }
}

private func rustCall<T>(_ callback: (UnsafeMutablePointer<RustCallStatus>) -> T) throws -> T {
    try makeRustCall(callback, errorHandler: {
        $0.deallocate()
        return UniffiInternalError.unexpectedRustCallError
    })
}

private func rustCallWithError<T, E: ViaFfiUsingByteBuffer & Error>(_ errorClass: E.Type, _ callback: (UnsafeMutablePointer<RustCallStatus>) -> T) throws -> T {
    try makeRustCall(callback, errorHandler: { return try E.lift($0) })
}

private func makeRustCall<T>(_ callback: (UnsafeMutablePointer<RustCallStatus>) -> T, errorHandler: (RustBuffer) throws -> Error) throws -> T {
    var callStatus = RustCallStatus.init()
    let returnedVal = callback(&callStatus)
    switch callStatus.code {
        case CALL_SUCCESS:
            return returnedVal

        case CALL_ERROR:
            throw try errorHandler(callStatus.errorBuf)

        case CALL_PANIC:
            // When the rust code sees a panic, it tries to construct a RustBuffer
            // with the message.  But if that code panics, then it just sends back
            // an empty buffer.
            if callStatus.errorBuf.len > 0 {
                throw UniffiInternalError.rustPanic(try String.lift(callStatus.errorBuf))
            } else {
                callStatus.errorBuf.deallocate()
                throw UniffiInternalError.rustPanic("Rust panic")
            }

        default:
            throw UniffiInternalError.unexpectedRustCallStatusCode
    }
}


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
    private var counter: [Handle: UInt64] = [:]
    private var rightMap: [ObjectIdentifier: Handle] = [:]

    private let lock = Lock()
    private var currentHandle: Handle = 0
    private let stride: Handle = 1

    func insert(obj: T) -> Handle {
        lock.withLock {
            let handle = rightMap[ObjectIdentifier(obj)] ?? {
                currentHandle += stride
                let handle = currentHandle
                leftMap[handle] = obj
                rightMap[ObjectIdentifier(obj)] = handle
                return handle
            }()
            counter[handle] = (counter[handle] ?? 0) + 1
            return handle
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
            defer { counter[handle] = (counter[handle] ?? 1) - 1 }
            guard counter[handle] == 1 else { return leftMap[handle] }
            let obj = leftMap.removeValue(forKey: handle)
            if let obj = obj {
                rightMap.removeValue(forKey: ObjectIdentifier(obj))
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
