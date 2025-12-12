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
internal inline fun<T> uniffiTraitInterfaceCallAsync(
    crossinline makeCall: suspend () -> T,
    crossinline handleSuccess: (T) -> Unit,
    crossinline handleError: (UniffiRustCallStatus.ByValue) -> Unit,
    uniffiOutDroppedCallback: UniffiForeignFutureDroppedCallbackStruct,
) {
    // Using `GlobalScope` is labeled as a "delicate API" and generally discouraged in Kotlin programs, since it breaks structured concurrency.
    // However, our parent task is a Rust future, so we're going to need to break structure concurrency in any case.
    //
    // Uniffi does its best to support structured concurrency across the FFI.
    // If the Rust future is dropped, `uniffiForeignFutureDroppedCallbackImpl` is called, which will cancel the Kotlin coroutine if it's still running.
    @OptIn(DelicateCoroutinesApi::class)
    val job = GlobalScope.launch coroutineBlock@ {
        // Note: it's important we call either `handleSuccess` or `handleError` exactly once.  Each
        // call consumes an Arc reference, which means there should be no possibility of a double
        // call.  The following code is structured so that will will never call both `handleSuccess`
        // and `handleError`, even in the face of weird exceptions.
        //
        // In extreme circumstances we may not call either, for example if we fail to make the JNA
        // call to `handleSuccess`.  This means we will leak the Arc reference, which is better than
        // double-freeing it.
        val callResult = try {
            makeCall()
        } catch(e: kotlin.Exception) {
            handleError(
                UniffiRustCallStatus.create(
                    UNIFFI_CALL_UNEXPECTED_ERROR,
                    {{ Type::String.borrow()|lower_fn }}(e.toString()),
                )
            )
            return@coroutineBlock
        }
        handleSuccess(callResult)
    }
    val handle = uniffiForeignFutureHandleMap.insert(job)
    uniffiOutDroppedCallback.uniffiSetValue(UniffiForeignFutureDroppedCallbackStruct(handle, uniffiForeignFutureDroppedCallbackImpl))
}

internal inline fun<T, reified E: Throwable> uniffiTraitInterfaceCallAsyncWithError(
    crossinline makeCall: suspend () -> T,
    crossinline handleSuccess: (T) -> Unit,
    crossinline handleError: (UniffiRustCallStatus.ByValue) -> Unit,
    crossinline lowerError: (E) -> RustBuffer.ByValue,
    uniffiOutDroppedCallback: UniffiForeignFutureDroppedCallbackStruct,
) {
    // See uniffiTraitInterfaceCallAsync for details on `DelicateCoroutinesApi`
    @OptIn(DelicateCoroutinesApi::class)
    val job = GlobalScope.launch coroutineBlock@ {
        // See the note in uniffiTraitInterfaceCallAsync for details on `handleSuccess` and
        // `handleError`.
        val callResult = try {
            makeCall()
        } catch(e: kotlin.Exception) {
            if (e is E) {
                handleError(
                    UniffiRustCallStatus.create(
                        UNIFFI_CALL_ERROR,
                        lowerError(e),
                    )
                )
            } else {
                handleError(
                    UniffiRustCallStatus.create(
                        UNIFFI_CALL_UNEXPECTED_ERROR,
                        {{ Type::String.borrow()|lower_fn }}(e.toString()),
                    )
                )
            }
            return@coroutineBlock
        }
        handleSuccess(callResult)
    }
    val handle = uniffiForeignFutureHandleMap.insert(job)
    uniffiOutDroppedCallback.uniffiSetValue(UniffiForeignFutureDroppedCallbackStruct(handle, uniffiForeignFutureDroppedCallbackImpl))
}

internal val uniffiForeignFutureHandleMap = UniffiHandleMap<Job>()

internal object uniffiForeignFutureDroppedCallbackImpl: UniffiForeignFutureDroppedCallback {
    override fun callback(handle: Long) {
        val job = uniffiForeignFutureHandleMap.remove(handle)
        if (!job.isCompleted) {
            job.cancel()
        }
    }
}

// For testing
public fun uniffiForeignFutureHandleCount() = uniffiForeignFutureHandleMap.size

{%- endif %}
