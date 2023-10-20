// Async return type handlers

internal const val UNIFFI_RUST_FUTURE_POLL_READY = 0.toShort()
internal const val UNIFFI_RUST_FUTURE_POLL_MAYBE_READY = 1.toShort()

internal val uniffiContinuationHandleMap = UniFfiHandleMap<CancellableContinuation<Short>>()

// FFI type for Rust future continuations
internal object uniffiRustFutureContinuationCallback: UniFffiRustFutureContinuationCallbackType {
    override fun callback(continuationHandle: USize, pollResult: Short) {
        uniffiContinuationHandleMap.remove(continuationHandle)?.resume(pollResult)
    }
}

internal suspend fun<T, F, E: Exception> uniffiRustCallAsync(
    rustFuture: UniffiHandle,
    pollFunc: (UniffiHandle, UniFffiRustFutureContinuationCallbackType, USize) -> Unit,
    completeFunc: (UniffiHandle, RustCallStatus) -> F,
    freeFunc: (UniffiHandle) -> Unit,
    liftFunc: (F) -> T,
    errorHandler: CallStatusErrorHandler<E>
): T {
    try {
        do {
            val pollResult = suspendCancellableCoroutine<Short> { continuation ->
                pollFunc(
                    rustFuture,
                    uniffiRustFutureContinuationCallback,
                    uniffiContinuationHandleMap.insert(continuation)
                )
            }
        } while (pollResult != UNIFFI_RUST_FUTURE_POLL_READY);

        return liftFunc(
            rustCallWithError(errorHandler, { status -> completeFunc(rustFuture, status) })
        )
    } finally {
        freeFunc(rustFuture)
    }
}

