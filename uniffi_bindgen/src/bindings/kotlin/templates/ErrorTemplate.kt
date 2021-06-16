{#-
// Here we define error conversions from native references to Kotlin exceptions.
// This is done by defining a RustErrorReference interface, where any implementers of the interface
// can be converted into a native reference for an error to flow across the FFI.

// A generic RustError is definied as a super class for other errors. This includes some common behaviour
// across the errors, can also serve as an error in case the cause of the error is unknown
-#}

interface RustErrorReference : Structure.ByReference {
    fun isFailure(): Boolean
    fun<E: Exception> intoException(): E
    fun ensureConsumed()
    fun getMessage(): String?
    fun consumeErrorMessage(): String
}

@Structure.FieldOrder("code", "message")
internal open class RustError : Structure() {
   open class ByReference: RustError(), RustErrorReference

    @JvmField var code: Int = 0
    @JvmField var message: Pointer? = null

    /**
     * Does this represent success?
     */
    fun isSuccess(): Boolean {
        return code == 0
    }

    /**
     * Does this represent failure?
     */
    fun isFailure(): Boolean {
        return code != 0
    }

    @Synchronized
    fun ensureConsumed() {
        if (this.message != null) {
            rustCall(InternalError.ByReference()) { err ->
                _UniFFILib.INSTANCE.{{ ci.ffi_string_free().name() }}(this.message!!, err)
             }
            this.message = null
        }
    }

    /**
     * Get the error message or null if there is none.
     */
    fun getMessage(): String? {
        return this.message?.getString(0, "utf8")
    }

    /**
     * Get and consume the error message, or null if there is none.
     */
    @Synchronized
    fun consumeErrorMessage(): String {
        val result = this.getMessage()
        if (this.message != null) {
            this.ensureConsumed()
        }
        if (result == null) {
            throw NullPointerException("consumeErrorMessage called with null message!")
        }
        return result
    }

    @Suppress("ReturnCount", "TooGenericExceptionThrown")
    open fun<E: Exception> intoException(): E {
        if (!isFailure()) {
            // It's probably a bad idea to throw here! We're probably leaking something if this is
            // ever hit! (But we shouldn't ever hit it?)
            throw RuntimeException("[Bug] intoException called on non-failure!")
        }
        this.consumeErrorMessage()
        throw RuntimeException("Generic errors are not implemented yet")
    }
}

internal open class InternalError : RustError() {
    class ByReference: InternalError(), RustErrorReference

    @Suppress("ReturnCount", "TooGenericExceptionThrown", "UNCHECKED_CAST")
    override fun<E: Exception> intoException(): E {
        if (!isFailure()) {
            // It's probably a bad idea to throw here! We're probably leaking something if this is
            // ever hit! (But we shouldn't ever hit it?)
            throw RuntimeException("[Bug] intoException called on non-failure!")
        }
        val message = this.consumeErrorMessage()
        return InternalException(message) as E
    }
}

class InternalException(message: String) : Exception(message)

{%- for e in ci.iter_error_definitions() %}
internal open class {{e.name()}} : RustError() {
    class ByReference: {{e.name()}}(), RustErrorReference

    @Suppress("ReturnCount", "TooGenericExceptionThrown", "UNCHECKED_CAST")
    override fun<E: Exception> intoException(): E {
        if (!isFailure()) {
            // It's probably a bad idea to throw here! We're probably leaking something if this is
            // ever hit! (But we shouldn't ever hit it?)
            throw RuntimeException("[Bug] intoException called on non-failure!")
        }
        val message = this.consumeErrorMessage()
        when (code) {
            {% for value in e.values() -%}
            {{loop.index}} -> return {{e.name()}}Exception.{{value}}(message) as E
            {% endfor -%}
            -1 -> return InternalException(message) as E
            else -> throw RuntimeException("invalid error code passed across the FFI")
        }
    }
}

open class {{e.name()}}Exception(message: String) : Exception(message) {
    {% for value in e.values() -%}
    class {{value}}(msg: String) : {{e.name()}}Exception(msg)
    {% endfor %}
}

{% endfor %}

// Helpers for calling Rust with errors:
// In practice we usually need to be synchronized to call this safely, so it doesn't
// synchronize itself
private inline fun <U, E: RustErrorReference> nullableRustCall(callback: (E) -> U?, err: E): U? {
    try {
        val ret = callback(err)
        if (err.isFailure()) {
            throw err.intoException()
        }
        return ret
    } finally {
        // This only matters if `callback` throws (or does a non-local return, which
        // we currently don't do)
        err.ensureConsumed()
    }
}

private inline fun <U, E: RustErrorReference> rustCall(err: E, callback: (E) -> U?): U {
    return nullableRustCall(callback, err)!!
}
