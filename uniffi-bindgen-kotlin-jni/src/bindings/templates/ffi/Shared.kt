@file:OptIn(kotlin.ExperimentalUnsignedTypes::class)
package uniffi

{% include "FfiBuffer.kt" %}
{% include "LiftLower.kt" %}
{% include "Interfaces.kt" %}
{% include "CallbackInterfaces.kt" %}

const val UNIFFI_RUST_FUTURE_PENDING = 0
const val UNIFFI_RUST_FUTURE_CANCELLED = 1
const val UNIFFI_RUST_FUTURE_COMPLETE = 2

class InternalException(message: kotlin.String) : Exception(message)

fun uniffiContinuationResume(continuation: kotlin.coroutines.Continuation<kotlin.Boolean>) {
    continuation.resumeWith(Result.success(false))
}

/**
  * Scaffolding function definitions shared by all packages
  */
object Scaffolding {
    init {
        System.loadLibrary("{{ cdylib }}")
        Scaffolding.ffiBufferCheckSupport()
    }

    @JvmStatic external fun ffiBufferCheckSupport()
    @JvmStatic external fun ffiBufferAlloc(capacity: kotlin.Int): java.nio.ByteBuffer
    @JvmStatic external fun ffiBufferFree(buf: java.nio.ByteBuffer)
    @JvmStatic external fun ffiBufferReadString(buf: java.nio.ByteBuffer, offset: kotlin.Int): kotlin.String
    @JvmStatic external fun ffiBufferWriteString(buf: java.nio.ByteBuffer, offset: kotlin.Int, value: kotlin.String)
    @JvmStatic external fun ffiBufferReadBytes(buf: java.nio.ByteBuffer, offset: kotlin.Int): kotlin.ByteArray
    @JvmStatic external fun ffiBufferWriteBytes(buf: java.nio.ByteBuffer, offset: kotlin.Int, value: kotlin.ByteArray)
    @JvmStatic external fun ffiBufferReadBuffer(buf: java.nio.ByteBuffer, offset: kotlin.Int): java.nio.ByteBuffer
    @JvmStatic external fun ffiBufferWriteBuffer(buf: java.nio.ByteBuffer, offset: kotlin.Int, value: java.nio.ByteBuffer)
    @JvmStatic external fun uniffiKotlinFutureComplete(kotlinFuture: kotlin.Long, resultCode: kotlin.Int)
}

{%- let type_definitions = root.non_user_type_definitions %}
{%- include "ffi/TypeDefinitions.kt" %}
