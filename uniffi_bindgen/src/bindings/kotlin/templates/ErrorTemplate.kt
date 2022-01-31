{% import "macros.kt" as kt %}
{%- let e = self.inner() %}
{%- let type_name = e|type_name -%}

{% if e.is_flat() %}
sealed class {{ type_name }}(message: String): Exception(message){% if self.contains_object_references() %}, Disposable {% endif %} {
        // Each variant is a nested class
        // Flat enums carries a string error message, so no special implementation is necessary.
        {% for variant in e.variants() -%}
        class {{ variant.name()|exception_name }}(message: String) : {{ type_name }}(message)
        {% endfor %}

    companion object ErrorHandler : CallStatusErrorHandler<{{ type_name }}> {
        override fun lift(error_buf: RustBuffer.ByValue): {{ type_name }} = {{ e|lift_fn }}(error_buf)
    }
}
{%- else %}
sealed class {{ type_name }}: Exception(){% if self.contains_object_references() %}, Disposable {% endif %} {
    // Each variant is a nested class
    {% for variant in e.variants() -%}
    {% if !variant.has_fields() -%}
    class {{ variant.name()|exception_name }} : {{ type_name }}()
    {% else %}
    class {{ variant.name()|exception_name }}(
        {% for field in variant.fields() -%}
        val {{ field.name()|var_name }}: {{ field|type_name}}{% if loop.last %}{% else %}, {% endif %}
        {% endfor -%}
    ) : {{ type_name }}()
    {%- endif %}
    {% endfor %}

    companion object ErrorHandler : CallStatusErrorHandler<{{ type_name }}> {
        override fun lift(error_buf: RustBuffer.ByValue): {{ type_name }} = {{ e|lift_fn }}(error_buf)
    }

    {% if self.contains_object_references() %}
    @Suppress("UNNECESSARY_SAFE_CALL") // codegen is much simpler if we unconditionally emit safe calls here
    override fun destroy() {
        when(this) {
            {%- for variant in e.variants() %}
            is {{ type_name }}.{{ variant.name()|class_name }} -> {
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
{%- endif %}

internal object {{ e|ffi_converter_name }} {
    fun lift(error_buf: RustBuffer.ByValue): {{ type_name }} {
        return liftFromRustBuffer(error_buf) { error_buf -> read(error_buf) }
    }

    fun read(error_buf: ByteBuffer): {{ type_name }} {
        {% if e.is_flat() %}
            return when(error_buf.getInt()) {
            {%- for variant in e.variants() %}
            {{ loop.index }} -> {{ type_name }}.{{ variant.name()|exception_name }}({{ TypeIdentifier::String|read_fn }}(error_buf))
            {%- endfor %}
            else -> throw RuntimeException("invalid error enum value, something is very wrong!!")
        }
        {% else %}

        return when(error_buf.getInt()) {
            {%- for variant in e.variants() %}
            {{ loop.index }} -> {{ type_name }}.{{ variant.name()|exception_name }}({% if variant.has_fields() %}
                {% for field in variant.fields() -%}
                {{ field|read_fn }}(error_buf),
                {% endfor -%}
            {%- endif -%})
            {%- endfor %}
            else -> throw RuntimeException("invalid error enum value, something is very wrong!!")
        }
        {%- endif %}
    }

    @Suppress("UNUSED_PARAMETER")
    fun lower(value: {{ type_name }}): RustBuffer.ByValue {
        throw RuntimeException("Lowering Errors is not supported")
    }

    @Suppress("UNUSED_PARAMETER")
    fun write(value: {{ type_name }}, buf: RustBufferBuilder) {
        throw RuntimeException("Writing Errors is not supported")
    }

}
