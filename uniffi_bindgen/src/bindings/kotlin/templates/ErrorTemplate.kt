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

// Each top-level error class has a companion object that can read the error from a rust buffer
interface ErrorReader<E> {
    fun read(buf: ByteBuffer): E;
}

{%- for e in ci.iter_error_definitions() %}

// Error {{ e.name() }}
{%- let toplevel_name=e.name()|exception_name_kt %}
open class {{ toplevel_name }}: Exception() {
    // Each variant is a nested class
    {% for variant in e.variants() -%}
    {% if !variant.has_fields() -%}
    class {{ variant.name()|class_name_kt }} : {{ toplevel_name }}()
    {% else %}
    class {{ variant.name()|class_name_kt }}(
        {% for field in variant.fields() -%}
        val {{ field.name()|var_name_kt }}: {{ field.type_()|type_kt}}{% if loop.last %}{% else %}, {% endif %}
        {% endfor -%}
    ) : {{ toplevel_name }}()
    {%- endif %}
    {% endfor %}

    companion object BufferReader : ErrorReader<{{ toplevel_name }}> {
        override fun read(buf: ByteBuffer): {{ toplevel_name }} {
            return when(buf.getInt()) {
                {%- for variant in e.variants() %}
                {{ loop.index }} -> {{ toplevel_name }}.{{ variant.name()|class_name_kt }}({% if variant.has_fields() %}
                    {% for field in variant.fields() -%}
                    {{ "buf"|read_kt(field.type_()) }}{% if loop.last %}{% else %},{% endif %}
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

// Call a rust function

// Call a rust function that returns a Result<>.  Pass in the Error class companion that corresponds to the Err
private inline fun <U, E: Exception> rustCallWithError(error_reader: ErrorReader<E>, callback: (RustCallStatus) -> U): U {
    var status = RustCallStatus();
    val ret = callback(status)
    if (status.isSuccess()) {
        return ret
    } else if (status.isError()) {
        throw liftFromRustBuffer(status.error_buf) { buf -> error_reader.read(buf) }
    } else if (status.isPanic()) {
        throw InternalException("Rust Panic")
    } else {
        throw InternalException("Unknown rust call status: $status.code")
    }
}

private inline fun <U> rustCall(callback: (RustCallStatus) -> U): U {
    var status = RustCallStatus();
    val ret = callback(status)
    if (status.isSuccess()) {
        return ret
    } else if (status.isError()) {
        RustBuffer.free(status.error_buf)
        throw InternalException("CALL_ERROR, but no ErrorReader specified")
    } else if (status.isPanic()) {
        throw InternalException("Rust Panic")
    } else {
        throw InternalException("Unknown rust call status: $status.code")
    }
}
