// A handful of classes and functions to support the generated data structures.
// This would be a good candidate for isolating in its own ffi-support lib.

internal const val UNIFFI_CALL_SUCCESS = 0.toByte()
internal const val UNIFFI_CALL_ERROR = 1.toByte()
internal const val UNIFFI_CALL_UNEXPECTED_ERROR = 2.toByte()

@Structure.FieldOrder("code", "error_buf")
internal open class UniffiRustCallStatus : Structure() {
    @JvmField var code: Byte = 0
    @JvmField var error_buf: RustBuffer.ByValue = RustBuffer.ByValue()

    class ByValue: UniffiRustCallStatus(), Structure.ByValue

    fun isSuccess(): Boolean {
        return code == UNIFFI_CALL_SUCCESS
    }

    fun isError(): Boolean {
        return code == UNIFFI_CALL_ERROR
    }

    fun isPanic(): Boolean {
        return code == UNIFFI_CALL_UNEXPECTED_ERROR
    }

    companion object {
        fun create(code: Byte, errorBuf: RustBuffer.ByValue): UniffiRustCallStatus.ByValue {
            val callStatus = UniffiRustCallStatus.ByValue()
            callStatus.code = code
            callStatus.error_buf = errorBuf
            return callStatus
        }
    }
}

class InternalException(message: String) : kotlin.Exception(message)

/**
 * Each top-level error class has a companion object that can lift the error from the call status's rust buffer
 *
 * @suppress
 */
interface UniffiRustCallStatusErrorHandler<E> {
    fun lift(error_buf: RustBuffer.ByValue): E;
}

/**
 * Per-thread pool of UniffiRustCallStatus instances to avoid allocating a new JNA Structure
 * on every UniFFI call. Supports reentrancy via a depth-indexed stack.
 */
private object UniffiCallStatusPool {
    private val stack = ThreadLocal.withInitial { mutableListOf<UniffiRustCallStatus.ByValue>() }
    private val depth = ThreadLocal.withInitial { 0 }

    fun acquire(): UniffiRustCallStatus.ByValue {
        val d = depth.get()
        val newDepth = d + 1
        depth.set(newDepth)
        val list = stack.get()
        if (newDepth > list.size) {
            list.add(UniffiRustCallStatus.ByValue())
        }
        val status = list[newDepth - 1]
        // Reset for reuse - keep nested Structure instance, only reset fields
        status.code = 0
        status.error_buf.capacity = 0
        status.error_buf.len = 0
        status.error_buf.data = null
        return status
    }

    fun release() {
        val d = depth.get()
        if (d > 0) {
            depth.set(d - 1)
        }
    }
}

// Helpers for calling Rust
// In practice we usually need to be synchronized to call this safely, so it doesn't
// synchronize itself

// Call a rust function that returns a Result<>.  Pass in the Error class companion that corresponds to the Err
private inline fun <U, E: kotlin.Exception> uniffiRustCallWithError(errorHandler: UniffiRustCallStatusErrorHandler<E>, callback: (UniffiRustCallStatus) -> U): U {
    val status = UniffiCallStatusPool.acquire()
    try {
        val return_value = callback(status)
        uniffiCheckCallStatus(errorHandler, status)
        return return_value
    } finally {
        // Ensure any owned error buffer is freed before reuse
        // uniffiCheckCallStatus will have lifted/freed on error; on success buffer is clean
        status.code = 0
        status.error_buf.capacity = 0
        status.error_buf.len = 0
        status.error_buf.data = null
        UniffiCallStatusPool.release()
    }
}

// Check UniffiRustCallStatus and throw an error if the call wasn't successful
private fun<E: kotlin.Exception> uniffiCheckCallStatus(errorHandler: UniffiRustCallStatusErrorHandler<E>, status: UniffiRustCallStatus) {
    if (status.isSuccess()) {
        return
    } else if (status.isError()) {
        val buf = requireNotNull(status.error_buf) { "UniffiRustCallStatus.error_buf must be initialized by JNA for error status" }
        throw errorHandler.lift(buf)
    } else if (status.isPanic()) {
        // when the rust code sees a panic, it tries to construct a rustbuffer
        // with the message.  but if that code panics, then it just sends back
        // an empty buffer.
        val buf = requireNotNull(status.error_buf) { "UniffiRustCallStatus.error_buf must be initialized by JNA for panic status" }
        if (buf.len > 0) {
            throw InternalException({{ Type::String.borrow()|lift_fn }}(buf))
        } else {
            throw InternalException("Rust panic")
        }
    } else {
        throw InternalException("Unknown rust call status: $status.code")
    }
}

/**
 * UniffiRustCallStatusErrorHandler implementation for times when we don't expect a CALL_ERROR
 *
 * @suppress
 */
object UniffiNullRustCallStatusErrorHandler: UniffiRustCallStatusErrorHandler<InternalException> {
    override fun lift(error_buf: RustBuffer.ByValue): InternalException {
        RustBuffer.free(error_buf)
        return InternalException("Unexpected CALL_ERROR")
    }
}

// Call a rust function that returns a plain value
private inline fun <U> uniffiRustCall(callback: (UniffiRustCallStatus) -> U): U {
    return uniffiRustCallWithError(UniffiNullRustCallStatusErrorHandler, callback)
}

internal inline fun<T> uniffiTraitInterfaceCall(
    callStatus: UniffiRustCallStatus,
    makeCall: () -> T,
    writeReturn: (T) -> Unit,
) {
    try {
        writeReturn(makeCall())
    } catch(e: kotlin.Throwable) {
        val err = try { e.stackTraceToString() } catch(_: Throwable) { "" }
        callStatus.code = UNIFFI_CALL_UNEXPECTED_ERROR
        callStatus.error_buf = {{ Type::String.borrow()|lower_fn }}(err)
    }
}

internal inline fun<T, reified E: Throwable> uniffiTraitInterfaceCallWithError(
    callStatus: UniffiRustCallStatus,
    makeCall: () -> T,
    writeReturn: (T) -> Unit,
    lowerError: (E) -> RustBuffer.ByValue
) {
    try {
        writeReturn(makeCall())
    } catch(e: kotlin.Throwable) {
        if (e is E) {
            callStatus.code = UNIFFI_CALL_ERROR
            callStatus.error_buf = lowerError(e)
        } else {
            val err = try { e.stackTraceToString() } catch(_: Throwable) { "" }
            callStatus.code = UNIFFI_CALL_UNEXPECTED_ERROR
            callStatus.error_buf = {{ Type::String.borrow()|lower_fn }}(err)
        }
    }
}
