// Async return type handlers

internal const val UNIFFI_RUST_FUTURE_POLL_READY = 0.toByte()
internal const val UNIFFI_RUST_FUTURE_POLL_WAKE = 1.toByte()

internal val uniffiContinuationHandleMap = UniffiHandleMap<CancellableContinuation<Byte>>()

// FFI type for Rust future continuations
internal object uniffiRustFutureContinuationCallbackImpl: UniffiCallbackFunction {
    override fun callback(uniffiFfiBuffer: Pointer) {
        val uniffiArgsCursor = UniffiBufferCursor(uniffiFfiBuffer)
        val data = UniffiFfiSerializerLong.read(uniffiArgsCursor);
        val pollResult = UniffiFfiSerializerByte.read(uniffiArgsCursor);
        uniffiContinuationHandleMap.remove(data).resume(pollResult)
    }
}

private suspend fun<T> uniffiDriveRustFutureToCompletion(
    rustFuture: Long,
    returnValuesBufSize: Long,
    readReturnBuf: (UniffiBufferCursor) -> T,
): T {
    try {
        do {
            val pollResult = suspendCancellableCoroutine<Byte> { continuation ->
                val continuationHandle = uniffiContinuationHandleMap.insert(continuation)

                val pollFfiBuffer = Memory(24)
                val pollArgCursor = UniffiBufferCursor(pollFfiBuffer)
                UniffiFfiSerializerHandle.write(pollArgCursor, rustFuture)
                UniffiFfiSerializerBoundCallback.write(
                    pollArgCursor,
                    UniffiBoundCallback(uniffiRustFutureContinuationCallbackImpl, continuationHandle)
                )
                UniffiLib.{{ ci.pointer_ffi_rust_future_poll() }}(pollFfiBuffer)
            }
        } while (pollResult != UNIFFI_RUST_FUTURE_POLL_READY);

        val completeFfiBuffer = Memory(max(8, returnValuesBufSize))
        val completeArgCursor = UniffiBufferCursor(completeFfiBuffer)
        UniffiFfiSerializerHandle.write(completeArgCursor, rustFuture)
        UniffiLib.{{ ci.pointer_ffi_rust_future_complete() }}(completeFfiBuffer)
        return readReturnBuf(UniffiBufferCursor(completeFfiBuffer))
    } finally {
        val freeFfiBuffer = Memory(8)
        val freeArgCursor = UniffiBufferCursor(freeFfiBuffer)
        UniffiFfiSerializerHandle.write(freeArgCursor, rustFuture)
        UniffiLib.{{ ci.pointer_ffi_rust_future_free() }}(freeFfiBuffer)
    }
}

{%- if ci.has_async_callback_interface_definition() %}
// Launch an async callback method in a suspend scope and serialize the result to the ffi buffer
private inline fun uniffiAsyncCallback(
    uniffiFfiBuffer: Pointer,
    crossinline block: suspend () -> Unit
) {
    // Using `GlobalScope` is labeled as a "delicate API" and generally discouraged in Kotlin programs, since it breaks structured concurrency.
    // However, our parent task is a Rust future, so we're going to need to break structure concurrency in any case.
    //
    // Uniffi does its best to support structured concurrency across the FFI.
    // If the Rust future is dropped, `uniffiForeignFutureDroppedCallbackImpl` is called, which will cancel the Kotlin coroutine if it's still running.
    //
    // Returns the handle 
    @OptIn(DelicateCoroutinesApi::class)
    val job = GlobalScope.launch {
        block()
    }
    val handle = uniffiForeignFutureHandleMap.insert(job)
    val returnCursor = UniffiBufferCursor(uniffiFfiBuffer)
    UniffiFfiSerializerForeignFutureDroppedCallback.write(
        returnCursor,
        UniffiBoundCallback(uniffiForeignFutureDroppedCallbackImpl, handle)
    )
}

internal val uniffiForeignFutureHandleMap = UniffiHandleMap<Job>()

internal object uniffiForeignFutureDroppedCallbackImpl: UniffiCallbackFunction {
    override fun callback(uniffiFfiBuffer: Pointer) {
        val argCursor = UniffiBufferCursor(uniffiFfiBuffer)
        val handle = UniffiFfiSerializerHandle.read(argCursor)
        val job = uniffiForeignFutureHandleMap.remove(handle)
        if (!job.isCompleted) {
            job.cancel()
        }
    }
}

// For testing
public fun uniffiForeignFutureHandleCount() = uniffiForeignFutureHandleMap.size

{%- endif %}
