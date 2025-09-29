private let UNIFFI_RUST_FUTURE_POLL_READY: Int8 = 0
private let UNIFFI_RUST_FUTURE_POLL_WAKE: Int8 = 1

fileprivate let uniffiContinuationHandleMap = UniffiHandleMap<UnsafeContinuation<Int8, Never>>()

fileprivate func uniffiRustCallAsync<F, T>(
    rustFutureFunc: () -> UInt64,
    pollFunc: (UInt64, @escaping UniffiRustFutureContinuationCallback, UInt64) -> (),
    completeFunc: (UInt64, UnsafeMutablePointer<RustCallStatus>) -> F,
    freeFunc: (UInt64) -> (),
    liftFunc: (F) throws -> T,
    errorHandler: ((RustBuffer) throws -> Swift.Error)?
) async throws -> T {
    // Make sure to call the ensure init function since future creation doesn't have a
    // RustCallStatus param, so doesn't use makeRustCall()
    {{ ensure_init_fn_name }}()
    let rustFuture = rustFutureFunc()
    defer {
        freeFunc(rustFuture)
    }
    var pollResult: Int8;
    repeat {
        pollResult = await withUnsafeContinuation {
            pollFunc(
                rustFuture,
                { handle, pollResult in
                    uniffiFutureContinuationCallback(handle: handle, pollResult: pollResult)
                },
                uniffiContinuationHandleMap.insert(obj: $0)
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
fileprivate func uniffiFutureContinuationCallback(handle: UInt64, pollResult: Int8) {
    if let continuation = try? uniffiContinuationHandleMap.remove(handle: handle) {
        continuation.resume(returning: pollResult)
    } else {
        print("uniffiFutureContinuationCallback invalid handle")
    }
}

{%- if ci.has_async_callback_interface_definition() %}
private func uniffiTraitInterfaceCallAsync<T>(
    makeCall: @escaping () async throws -> T,
    handleSuccess: @escaping (T) -> (),
    handleError: @escaping (Int8, RustBuffer) -> (),
    droppedCallback: UnsafeMutablePointer<UniffiForeignFutureDroppedCallbackStruct>
) {
    let task = Task {
        do {
            handleSuccess(try await makeCall())
        } catch {
            handleError(CALL_UNEXPECTED_ERROR, {{ Type::String.borrow()|lower_fn }}(String(describing: error)))
        }
    }
    let handle = UNIFFI_FOREIGN_FUTURE_HANDLE_MAP.insert(obj: task)
    droppedCallback.pointee = UniffiForeignFutureDroppedCallbackStruct(
        handle: handle,
        free: uniffiForeignFutureDroppedCallback
    )
}

private func uniffiTraitInterfaceCallAsyncWithError<T, E>(
    makeCall: @escaping () async throws -> T,
    handleSuccess: @escaping (T) -> (),
    handleError: @escaping (Int8, RustBuffer) -> (),
    lowerError: @escaping (E) -> RustBuffer,
    droppedCallback: UnsafeMutablePointer<UniffiForeignFutureDroppedCallbackStruct>
) {
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
    droppedCallback.pointee = UniffiForeignFutureDroppedCallbackStruct(
        handle: handle,
        free: uniffiForeignFutureDroppedCallback
    )
}

// Borrow the callback handle map implementation to store foreign future handles
// TODO: consolidate the handle-map code (https://github.com/mozilla/uniffi-rs/pull/1823)
fileprivate let UNIFFI_FOREIGN_FUTURE_HANDLE_MAP = UniffiHandleMap<UniffiForeignFutureTask>()

// Protocol for tasks that handle foreign futures.
//
// Defining a protocol allows all tasks to be stored in the same handle map.  This can't be done
// with the task object itself, since has generic parameters.
fileprivate protocol UniffiForeignFutureTask {
    func cancel()
}

extension Task: UniffiForeignFutureTask {}

private func uniffiForeignFutureDroppedCallback(handle: UInt64) {
    do {
        let task = try UNIFFI_FOREIGN_FUTURE_HANDLE_MAP.remove(handle: handle)
        // Set the cancellation flag on the task.  If it's still running, the code can check the
        // cancellation flag or call `Task.checkCancellation()`.  If the task has completed, this is
        // a no-op.
        task.cancel()
    } catch {
        print("uniffiForeignFutureDroppedCallback: handle missing from handlemap")
    }
}

// For testing
public func uniffiForeignFutureHandleCount{{ ci.namespace()|class_name }}() -> Int {
    UNIFFI_FOREIGN_FUTURE_HANDLE_MAP.count
}

{%- endif %}
