// Async return type handlers

internal const val UNIFFI_RUST_FUTURE_POLL_READY = 0.toByte()
internal const val UNIFFI_RUST_FUTURE_POLL_MAYBE_READY = 1.toByte()

internal val uniffiContinuationHandleMap = UniFfiHandleMap<CancellableContinuation<Byte>>()

// FFI type for Rust future continuations
internal object uniffiRustFutureContinuationCallback: UniffiRustFutureContinuationCallback {
    override fun callback(data: USize, pollResult: Byte) {
        uniffiContinuationHandleMap.remove(data)?.resume(pollResult)
    }
}

internal suspend fun<T, F, E: Exception> uniffiRustCallAsync(
    rustFuture: Pointer,
    pollFunc: (Pointer, UniffiRustFutureContinuationCallback, USize) -> Unit,
    completeFunc: (Pointer, RustCallStatus) -> F,
    freeFunc: (Pointer) -> Unit,
    liftFunc: (F) -> T,
    errorHandler: CallStatusErrorHandler<E>
): T {
    try {
        do {
            val pollResult = suspendCancellableCoroutine<Byte> { continuation ->
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

