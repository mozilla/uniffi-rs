// Async return type handlers

internal const val UNIFFI_RUST_FUTURE_POLL_READY = 0.toByte()
internal const val UNIFFI_RUST_FUTURE_POLL_MAYBE_READY = 1.toByte()

/**
 * Data for an in-progress poll of a RustFuture
 */
internal data class UniffiPollData(
    val continuation: CancellableContinuation<Byte>,
    val rustFuture: Long,
    val pollFunc: (Long, UniffiRustFutureContinuationCallback, Long, Long) -> Unit,
)

// Stores the UniffiPollData instances that correspond to RustFuture callback data
internal val uniffiPollDataHandleMap = UniffiHandleMap<UniffiPollData>()

// Stores the CoroutineContext instances that correspond to blocking task queue handles
internal val uniffiBlockingTaskQueueHandleMap = UniffiHandleMap<CoroutineContext>()

// FFI type for Rust future continuations
internal object uniffiRustFutureContinuationCallbackImpl: UniffiRustFutureContinuationCallback {
    override fun callback(data: Long, pollResult: Byte, blockingTaskQueueHandle: Long) {
        if (blockingTaskQueueHandle == 0L) {
            // Complete the Kotlin continuation
            uniffiPollDataHandleMap.remove(data).continuation.resume(pollResult)
        } else {
            // Call the poll function again, but inside the BlockingTaskQueue coroutine context
            val coroutineContext = uniffiBlockingTaskQueueHandleMap.get(blockingTaskQueueHandle)
            val pollData = uniffiPollDataHandleMap.get(data)
            CoroutineScope(coroutineContext).launch {
                pollData.pollFunc(pollData.rustFuture, uniffiRustFutureContinuationCallbackImpl, data, blockingTaskQueueHandle)
            }
        }
    }
}

internal suspend fun<T, F, E: kotlin.Exception> uniffiRustCallAsync(
    rustFuture: Long,
    pollFunc: (Long, UniffiRustFutureContinuationCallback, Long, Long) -> Unit,
    completeFunc: (Long, UniffiRustCallStatus) -> F,
    freeFunc: (Long) -> Unit,
    liftFunc: (F) -> T,
    errorHandler: UniffiRustCallStatusErrorHandler<E>
): T {
    try {
        do {
            val pollResult = suspendCancellableCoroutine<Byte> { continuation ->
                val pollData = UniffiPollData(continuation, rustFuture, pollFunc)
                pollFunc(
                    rustFuture,
                    uniffiRustFutureContinuationCallbackImpl,
                    uniffiPollDataHandleMap.insert(pollData),
                    0L
                )
            }
        } while (pollResult != UNIFFI_RUST_FUTURE_POLL_READY);

        return liftFunc(
            uniffiRustCallWithError(errorHandler, { status -> completeFunc(rustFuture, status) })
        )
    } finally {
        freeFunc(rustFuture)
    }
}

{%- if ci.has_async_callback_interface_definition() %}
internal inline fun<T> uniffiTraitInterfaceCallAsync(
    crossinline makeCall: suspend () -> T,
    crossinline handleSuccess: (T) -> Unit,
    crossinline handleError: (UniffiRustCallStatus.ByValue) -> Unit,
): UniffiForeignFuture {
    // Using `GlobalScope` is labeled as a "delicate API" and generally discouraged in Kotlin programs, since it breaks structured concurrency.
    // However, our parent task is a Rust future, so we're going to need to break structure concurrency in any case.
    //
    // Uniffi does its best to support structured concurrency across the FFI.
    // If the Rust future is dropped, `uniffiForeignFutureFreeImpl` is called, which will cancel the Kotlin coroutine if it's still running.
    @OptIn(DelicateCoroutinesApi::class)
    val job = GlobalScope.launch {
        try {
            handleSuccess(makeCall())
        } catch(e: Exception) {
            handleError(
                UniffiRustCallStatus.create(
                    UNIFFI_CALL_UNEXPECTED_ERROR,
                    {{ Type::String.borrow()|lower_fn }}(e.toString()),
                )
            )
        }
    }
    val handle = uniffiForeignFutureHandleMap.insert(job)
    return UniffiForeignFuture(handle, uniffiForeignFutureFreeImpl)
}

internal inline fun<T, reified E: Throwable> uniffiTraitInterfaceCallAsyncWithError(
    crossinline makeCall: suspend () -> T,
    crossinline handleSuccess: (T) -> Unit,
    crossinline handleError: (UniffiRustCallStatus.ByValue) -> Unit,
    crossinline lowerError: (E) -> RustBuffer.ByValue,
): UniffiForeignFuture {
    // See uniffiTraitInterfaceCallAsync for details on `DelicateCoroutinesApi`
    @OptIn(DelicateCoroutinesApi::class)
    val job = GlobalScope.launch {
        try {
            handleSuccess(makeCall())
        } catch(e: Exception) {
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
        }
    }
    val handle = uniffiForeignFutureHandleMap.insert(job)
    return UniffiForeignFuture(handle, uniffiForeignFutureFreeImpl)
}

internal val uniffiForeignFutureHandleMap = UniffiHandleMap<Job>()

internal object uniffiForeignFutureFreeImpl: UniffiForeignFutureFree {
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

// For testing
public fun uniffiPollHandleCount() = uniffiPollDataHandleMap.size
