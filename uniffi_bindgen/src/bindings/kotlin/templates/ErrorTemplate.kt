{% import "macros.kt" as kt %}
{%- let e = self.inner() %}
{%- let type_ = e.type_() %}

// Error {{ e.name() }}
{%- let toplevel_name=e.name()|exception_name_kt %}
{% if e.is_flat() %}
sealed class {{ toplevel_name }}(message: String): Exception(message){% if self.contains_object_references() %}, Disposable {% endif %} {
        // Each variant is a nested class
        // Flat enums carries a string error message, so no special implementation is necessary.
        {% for variant in e.variants() -%}
        class {{ variant.name()|exception_name_kt }}(message: String) : {{ toplevel_name }}(message)
        {% endfor %}

{%- else %}
sealed class {{ toplevel_name }}: Exception(){% if self.contains_object_references() %}, Disposable {% endif %} {
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

{%- endif %}
    companion object ErrorHandler : CallStatusErrorHandler<{{ toplevel_name }}> {
        override fun lift(error_buf: RustBuffer.ByValue) = {{ type_|ffi_converter_name }}.lift(error_buf)
    }

    {% if self.contains_object_references() %}
    @Suppress("UNNECESSARY_SAFE_CALL") // codegen is much simpler if we unconditionally emit safe calls here
    override fun destroy() {
        when(this) {
            {%- for variant in e.variants() %}
            is {{ e.name()|class_name_kt }}.{{ variant.name()|class_name_kt }} -> {
                {%- if variant.has_fields() %}
                {% call kt::destroy_fields(variant) %}
                {% else -%}
                // Nothing to destroy
                {%- endif %}
            }
            {%- endfor %}
        }.let { /* this makes the `when` an expression, which ensures it is exhaustive */ }
    }
    {% endif %}
}

object {{ type_|ffi_converter_name }} {
    fun read(error_buf: ByteBuffer): {{ toplevel_name }} {
        {% if e.is_flat() %}
            return when(error_buf.getInt()) {
            {%- for variant in e.variants() %}
            {{ loop.index }} -> {{ toplevel_name }}.{{ variant.name()|exception_name_kt }}(FFIConverterString.read(error_buf))
            {%- endfor %}
            else -> throw RuntimeException("invalid error enum value, something is very wrong!!")
        }
        {% else %}

        return when(error_buf.getInt()) {
            {%- for variant in e.variants() %}
            {{ loop.index }} -> {{ toplevel_name }}.{{ variant.name()|exception_name_kt }}({% if variant.has_fields() %}
                {% for field in variant.fields() -%}
                {{ field.type_()|ffi_converter_name }}.read(error_buf){% if loop.last %}{% else %},{% endif %}
                {% endfor -%}
            {%- endif -%})
            {%- endfor %}
            else -> throw RuntimeException("invalid error enum value, something is very wrong!!")
        }
        {%- endif %}
    }

    @Suppress("UNUSED_PARAMETER")
    fun write(v: {{ toplevel_name }}, buf: RustBufferBuilder) {
        throw RuntimeException("writing/lowering errors is not supported")
    }

    {% call kt::lift_and_lower_from_read_and_write(type_) %}
}

