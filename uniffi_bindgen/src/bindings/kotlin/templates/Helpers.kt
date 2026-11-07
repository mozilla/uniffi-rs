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
 * Reusable native memory for the `RustCallStatus` out-parameter of Kotlin -> Rust calls.
 *
 * Every FFI function that can fail takes a pointer to a `RustCallStatus`, which Rust writes to
 * only when the call fails. Passing a `UniffiRustCallStatus` JNA `Structure` for it means
 * allocating the Structure (and its nested `RustBuffer` and native `Memory`) on every call, and
 * JNA copying it to and from native memory by reflection before and after the call. That costs
 * far more than the call itself. Instead, FFI functions take the status as a plain `Pointer` to
 * this buffer, and a `Structure` is only built when Rust reports an error.
 *
 * Rust expects the caller to have set the status to success before the call, and leaves it alone
 * when the call succeeds. One buffer per thread is enough, even when a call re-enters Rust from a
 * callback interface on the same thread, as long as whoever reads a failure also resets the
 * buffer (`takeFailure`). The buffer is then all zeros at every moment except between Rust
 * writing a failure and the caller reading it, so an inner call's failure can't become the
 * result of the call it's nested in, and a successful call reads the zeros Rust left untouched.
 *
 * Thread safety: each thread has its own buffer (a `ThreadLocal`), and Rust only writes to the
 * status during the call, on the calling thread, so a buffer is never shared between threads.
 *
 * @suppress
 */
internal object UniffiRustCallStatusBuffer {
    private val size: Int = UniffiRustCallStatus().size()

    // Not `ThreadLocal.withInitial`, which needs Android API level 26.
    private val buffer = object : ThreadLocal<com.sun.jna.Memory>() {
        override fun initialValue() = com.sun.jna.Memory(size.toLong()).also { it.clear() }
    }

    fun get(): Pointer = buffer.get()

    /** Copy a failed call's status out for the error handlers and reset the buffer to success. */
    fun takeFailure(buffer: Pointer): UniffiRustCallStatus {
        val status = UniffiRustCallStatus()
        status.pointer.write(0, buffer.getByteArray(0, size), 0, size)
        status.read()
        buffer.setByte(0, UNIFFI_CALL_SUCCESS)
        return status
    }
}

// Helpers for calling Rust
// In practice we usually need to be synchronized to call this safely, so it doesn't
// synchronize itself

// Call a rust function that returns a Result<>.  Pass in the Error class companion that corresponds to the Err
//
// `callback` receives a pointer to the `RustCallStatus` to pass to the FFI function.
private inline fun <U, E: kotlin.Exception> uniffiRustCallWithError(errorHandler: UniffiRustCallStatusErrorHandler<E>, callback: (Pointer) -> U): U {
    val status = UniffiRustCallStatusBuffer.get()
    val return_value = callback(status)
    // `code` is the first field, a single byte.
    if (status.getByte(0) != UNIFFI_CALL_SUCCESS) {
        uniffiHandleCallError(errorHandler, UniffiRustCallStatusBuffer.takeFailure(status))
    }
    return return_value
}

// Throw the exception for a failed call's UniffiRustCallStatus
private fun<E: kotlin.Exception> uniffiHandleCallError(errorHandler: UniffiRustCallStatusErrorHandler<E>, status: UniffiRustCallStatus): Nothing {
    if (status.isError()) {
        throw errorHandler.lift(status.error_buf)
    } else if (status.isPanic()) {
        // when the rust code sees a panic, it tries to construct a rustbuffer
        // with the message.  but if that code panics, then it just sends back
        // an empty buffer.
        if (status.error_buf.len > 0) {
            throw InternalException({{ Type::String.borrow()|lift_fn }}(status.error_buf))
        } else {
            throw InternalException("Rust panic")
        }
    } else {
        throw InternalException("Unknown rust call status: ${status.code}")
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
private inline fun <U> uniffiRustCall(callback: (Pointer) -> U): U {
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
