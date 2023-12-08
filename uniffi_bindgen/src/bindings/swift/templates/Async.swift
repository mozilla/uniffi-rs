private let UNIFFI_RUST_FUTURE_POLL_READY: Int8 = 0
private let UNIFFI_RUST_FUTURE_POLL_MAYBE_READY: Int8 = 1

// Data for an in-progress poll of a RustFuture
fileprivate class UniffiPollData {
    let continuation: UnsafeContinuation<Int8, Never>
    let rustFuture: UInt64
    let pollFunc: (UInt64, @escaping UniffiRustFutureContinuationCallback, UInt64, UInt64) -> ()

    init(
        continuation: UnsafeContinuation<Int8, Never>,
        rustFuture: UInt64,
        pollFunc: @escaping (UInt64, @escaping UniffiRustFutureContinuationCallback, UInt64, UInt64) -> ()
    ) {
        self.continuation = continuation
        self.rustFuture = rustFuture
        self.pollFunc = pollFunc
    }
}

// Stores the UniffiPollData instances that correspond to RustFuture callback data
fileprivate let uniffiPollDataHandleMap = UniffiHandleMap<UniffiPollData>()

// Stores the DispatchQueue instances that correspond to blocking task queue handles
fileprivate var uniffiBlockingTaskQueueHandleMap = UniffiHandleMap<DispatchQueue>()

fileprivate func uniffiRustCallAsync<F, T>(
    rustFutureFunc: () -> UInt64,
    pollFunc: @escaping (UInt64, @escaping UniffiRustFutureContinuationCallback, UInt64, UInt64) -> (),
    completeFunc: (UInt64, UnsafeMutablePointer<RustCallStatus>) -> F,
    freeFunc: (UInt64) -> (),
    liftFunc: (F) throws -> T,
    errorHandler: ((RustBuffer) throws -> Error)?
) async throws -> T {
    // Make sure to call uniffiEnsureInitialized() since future creation doesn't have a
    // RustCallStatus param, so doesn't use makeRustCall()
    uniffiEnsureInitialized()
    let rustFuture = rustFutureFunc()
    defer {
        freeFunc(rustFuture)
    }
    var pollResult: Int8;
    repeat {
        pollResult = await withUnsafeContinuation {
            let pollData = UniffiPollData(
                continuation: $0,
                rustFuture: rustFuture,
                pollFunc: pollFunc
            )
            pollFunc(
                rustFuture,
                uniffiFutureContinuationCallback,
                uniffiPollDataHandleMap.insert(obj: pollData),
                0
            )

        }
    } while pollResult != UNIFFI_RUST_FUTURE_POLL_READY

    return try liftFunc(makeRustCall(
        { completeFunc(rustFuture, $0) },
        errorHandler: errorHandler
    ))
}

// Callback handlers for an async calls.  These are invoked by Rust when the future is ready.  They
// lift the return value or error and resume the suspended function.
fileprivate func uniffiFutureContinuationCallback(
    pollDataHandle: UInt64,
    pollResult: Int8,
    blockingTaskQueueHandle: UInt64
) {
    if (blockingTaskQueueHandle == 0) {
        // Try to complete the Swift continutation
        let pollData = try! uniffiPollDataHandleMap.remove(handle: pollDataHandle)
        pollData.continuation.resume(returning: pollResult)
    } else {
        // Call the poll function again, but inside the DispatchQuee
        let pollData = try! uniffiPollDataHandleMap.get(handle: pollDataHandle)
        let queue = try! uniffiBlockingTaskQueueHandleMap.get(handle: blockingTaskQueueHandle)
        queue.async {
            pollData.pollFunc(pollData.rustFuture, uniffiFutureContinuationCallback, pollDataHandle, blockingTaskQueueHandle)
        }
    }
}

{%- if ci.has_async_callback_interface_definition() %}
private func uniffiTraitInterfaceCallAsync<T>(
    makeCall: @escaping () async throws -> T,
    handleSuccess: @escaping (T) -> (),
    handleError: @escaping (Int8, RustBuffer) -> ()
) -> UniffiForeignFuture {
    let task = Task {
        do {
            handleSuccess(try await makeCall())
        } catch {
            handleError(CALL_UNEXPECTED_ERROR, {{ Type::String.borrow()|lower_fn }}(String(describing: error)))
        }
    }
    let handle = UNIFFI_FOREIGN_FUTURE_HANDLE_MAP.insert(obj: task)
    return UniffiForeignFuture(handle: handle, free: uniffiForeignFutureFree)

}

private func uniffiTraitInterfaceCallAsyncWithError<T, E>(
    makeCall: @escaping () async throws -> T,
    handleSuccess: @escaping (T) -> (),
    handleError: @escaping (Int8, RustBuffer) -> (),
    lowerError: @escaping (E) -> RustBuffer
) -> UniffiForeignFuture {
    let task = Task {
        do {
            handleSuccess(try await makeCall())
        } catch let error as E {
            handleError(CALL_ERROR, lowerError(error))
        } catch {
            handleError(CALL_UNEXPECTED_ERROR, {{ Type::String.borrow()|lower_fn }}(String(describing: error)))
        }
    }
    let handle = UNIFFI_FOREIGN_FUTURE_HANDLE_MAP.insert(obj: task)
    return UniffiForeignFuture(handle: handle, free: uniffiForeignFutureFree)
}

// Borrow the callback handle map implementation to store foreign future handles
// TODO: consolidate the handle-map code (https://github.com/mozilla/uniffi-rs/pull/1823)
fileprivate var UNIFFI_FOREIGN_FUTURE_HANDLE_MAP = UniffiHandleMap<UniffiForeignFutureTask>()

// Protocol for tasks that handle foreign futures.
//
// Defining a protocol allows all tasks to be stored in the same handle map.  This can't be done
// with the task object itself, since has generic parameters.
protocol UniffiForeignFutureTask {
    func cancel()
}

extension Task: UniffiForeignFutureTask {}

private func uniffiForeignFutureFree(handle: UInt64) {
    do {
        let task = try UNIFFI_FOREIGN_FUTURE_HANDLE_MAP.remove(handle: handle)
        // Set the cancellation flag on the task.  If it's still running, the code can check the
        // cancellation flag or call `Task.checkCancellation()`.  If the task has completed, this is
        // a no-op.
        task.cancel()
    } catch {
        print("uniffiForeignFutureFree: handle missing from handlemap")
    }
}

// For testing
public func uniffiForeignFutureHandleCount{{ ci.namespace()|class_name }}() -> Int {
    UNIFFI_FOREIGN_FUTURE_HANDLE_MAP.count
}

{%- endif %}

// For testing
public func uniffiPollDataHandleCount{{ ci.namespace()|class_name }}() -> Int {
    return uniffiPollDataHandleMap.count
}
