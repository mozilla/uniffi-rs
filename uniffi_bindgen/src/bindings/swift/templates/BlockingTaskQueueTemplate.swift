fileprivate var UNIFFI_BLOCKING_TASK_QUEUE_VTABLE = UniffiBlockingTaskQueueVTable(
    clone: { (handle: UInt64) -> UInt64 in
        do {
            let dispatchQueue = try uniffiBlockingTaskQueueHandleMap.get(handle: handle)
            return uniffiBlockingTaskQueueHandleMap.insert(obj: dispatchQueue)
        } catch {
            print("UniffiBlockingTaskQueueVTable.clone: invalid task queue handle")
            return 0
        }
    },
    free: { (handle: UInt64) in
        do {
            try uniffiBlockingTaskQueueHandleMap.remove(handle: handle)
        } catch {
            print("UniffiBlockingTaskQueueVTable.free: invalid task queue handle")
        }
    }
)

fileprivate struct {{ ffi_converter_name }}: FfiConverterRustBuffer {
    typealias SwiftType = DispatchQueue

    public static func write(_ value: DispatchQueue, into buf: inout [UInt8]) {
        let handle = uniffiBlockingTaskQueueHandleMap.insert(obj: value)
        writeInt(&buf, handle)
        // From Apple: "You can safely use the address of a global variable as a persistent unique
        // pointer value" (https://developer.apple.com/swift/blog/?id=6)
        let vtablePointer = UnsafeMutablePointer(&UNIFFI_BLOCKING_TASK_QUEUE_VTABLE)
        // Convert the pointer to a word-sized Int then to a 64-bit int then write it out.
        writeInt(&buf, Int64(Int(bitPattern: vtablePointer)))
    }

    public static func read(from buf: inout (data: Data, offset: Data.Index)) throws -> DispatchQueue {
        let handle: UInt64 = try readInt(&buf)
        // Read the VTable pointer and throw it out.  The vtable is only used by Rust and always the
        // same value.
        let _: UInt64 = try readInt(&buf)
        return try uniffiBlockingTaskQueueHandleMap.remove(handle: handle)
    }
}

// For testing
public func uniffiBlockingTaskQueueHandleCount{{ ci.namespace()|class_name }}() -> Int {
    uniffiBlockingTaskQueueHandleMap.count
}
