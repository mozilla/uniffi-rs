private let UNIFFI_RUST_FUTURE_POLL_READY: Int8 = 0
private let UNIFFI_RUST_FUTURE_POLL_MAYBE_READY: Int8 = 1

fileprivate var UNIFFI_CONTINUATION_SLAB = UniffiSlab<UnsafeContinuation<Int8, Never>>()

fileprivate func uniffiRustCallAsync<F, T>(
    rustFutureFunc: () -> Int64,
    pollFunc: (Int64, @escaping UniFfiRustFutureContinuation, Int64) -> (),
    completeFunc: (Int64, UnsafeMutablePointer<RustCallStatus>) -> F,
    freeFunc: (Int64) -> (),
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
            pollFunc(rustFuture, uniffiFutureContinuationCallback, try! UNIFFI_CONTINUATION_SLAB.insert(value: $0))
        }
    } while pollResult != UNIFFI_RUST_FUTURE_POLL_READY

    return try liftFunc(makeRustCall(
        { completeFunc(rustFuture, $0) },
        errorHandler: errorHandler
    ))
}

// Callback handlers for an async calls.  These are invoked by Rust when the future is ready.  They
// lift the return value or error and resume the suspended function.
fileprivate func uniffiFutureContinuationCallback(handle: Int64, pollResult: Int8) {
    try! UNIFFI_CONTINUATION_SLAB.remove(handle: handle).resume(returning: pollResult)
}
