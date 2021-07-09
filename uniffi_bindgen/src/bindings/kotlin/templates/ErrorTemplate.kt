@Structure.FieldOrder("code", "error_buf")
internal open class RustCallStatus : Structure() {
    @JvmField var code: Int = 0
    @JvmField var error_buf: RustBuffer.ByValue = RustBuffer.ByValue()

    fun isSuccess(): Boolean {
        return code == 0
    }

    fun isError(): Boolean {
        return code == 1
    }

    fun isPanic(): Boolean {
        return code == 2
    }
}

class InternalException(message: String) : Exception(message)

// Each top-level error class has a companion object that can lift the error from the call status's rust buffer
interface CallStatusErrorHandler<E> {
    fun lift(error_buf: RustBuffer.ByValue): E;
}

{%- for e in ci.iter_error_definitions() %}

// Error {{ e.name() }}
{%- let toplevel_name=e.name()|exception_name_kt %}
open class {{ toplevel_name }}: Exception() {
    // Each variant is a nested class
    {% for variant in e.variants() -%}
    {% if !variant.has_fields() -%}
    class {{ variant.name()|exception_name_kt }} : {{ toplevel_name }}()
    {% else %}
    class {{ variant.name()|exception_name_kt }}(
        {% for field in variant.fields() -%}
        val {{ field.name()|var_name_kt }}: {{ field.type_()|type_kt}}{% if loop.last %}{% else %}, {% endif %}
        {% endfor -%}
    ) : {{ toplevel_name }}()
    {%- endif %}
    {% endfor %}

    companion object ErrorHandler : CallStatusErrorHandler<{{ toplevel_name }}> {
        override fun lift(error_buf: RustBuffer.ByValue): {{ toplevel_name }} {
            return liftFromRustBuffer(error_buf) { error_buf -> read(error_buf) }
        }

        fun read(error_buf: ByteBuffer): {{ toplevel_name }} {
            return when(error_buf.getInt()) {
                {%- for variant in e.variants() %}
                {{ loop.index }} -> {{ toplevel_name }}.{{ variant.name()|exception_name_kt }}({% if variant.has_fields() %}
                    {% for field in variant.fields() -%}
                    {{ "error_buf"|read_kt(field.type_()) }}{% if loop.last %}{% else %},{% endif %}
                    {% endfor -%}
                {%- endif -%})
                {%- endfor %}
                else -> throw RuntimeException("invalid error enum value, something is very wrong!!")
            }
        }
    }
}
{% endfor %}

// Helpers for calling Rust
// In practice we usually need to be synchronized to call this safely, so it doesn't
// synchronize itself

// Call a rust function that returns a Result<>.  Pass in the Error class companion that corresponds to the Err
private inline fun <U, E: Exception> rustCallWithError(errorHandler: CallStatusErrorHandler<E>, callback: (RustCallStatus) -> U): U {
    var status = RustCallStatus();
    val return_value = callback(status)
    if (status.isSuccess()) {
        return return_value
    } else if (status.isError()) {
        throw errorHandler.lift(status.error_buf)
    } else if (status.isPanic()) {
        // when the rust code sees a panic, it tries to construct a rustbuffer
        // with the message.  but if that code panics, then it just sends back
        // an empty buffer.
        if (status.error_buf.len > 0) {
            throw InternalException(String.lift(status.error_buf))
        } else {
            throw InternalException("Rust panic")
        }
    } else {
        throw InternalException("Unknown rust call status: $status.code")
    }
}

// CallStatusErrorHandler implementation for times when we don't expect a CALL_ERROR
object NullCallStatusErrorHandler: CallStatusErrorHandler<InternalException> {
    override fun lift(error_buf: RustBuffer.ByValue): InternalException {
        RustBuffer.free(error_buf)
        return InternalException("Unexpected CALL_ERROR")
    }
}

// Call a rust function that returns a plain value
private inline fun <U> rustCall(callback: (RustCallStatus) -> U): U {
    return rustCallWithError(NullCallStatusErrorHandler, callback);
}
