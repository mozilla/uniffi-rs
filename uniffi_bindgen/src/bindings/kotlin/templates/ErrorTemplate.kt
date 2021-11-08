{% import "macros.kt" as kt %}
{%- let e = self.inner() %}

{% if e.is_flat() %}
sealed class {{ e|type_name }}(message: String): Exception(message){% if self.contains_object_references() %}, Disposable {% endif %} {
        // Each variant is a nested class
        // Flat enums carries a string error message, so no special implementation is necessary.
        {% for variant in e.variants() -%}
        class {{ variant.name()|exception_name }}(message: String) : {{ e|type_name }}(message)
        {% endfor %}

{%- else %}
sealed class {{ e|type_name }}: Exception(){% if self.contains_object_references() %}, Disposable {% endif %} {
    // Each variant is a nested class
    {% for variant in e.variants() -%}
    {% if !variant.has_fields() -%}
    class {{ variant.name()|exception_name }} : {{ e|type_name }}()
    {% else %}
    class {{ variant.name()|exception_name }}(
        {% for field in variant.fields() -%}
        val {{ field.name()|var_name }}: {{ field|type_name}}{% if loop.last %}{% else %}, {% endif %}
        {% endfor -%}
    ) : {{ e|type_name }}()
    {%- endif %}
    {% endfor %}

{%- endif %}

    companion object ErrorHandler : CallStatusErrorHandler<{{ e|type_name }}> {
        override fun lift(error_buf: RustBuffer.ByValue): {{ e|type_name }} {
            return liftFromRustBuffer(error_buf) { error_buf -> read(error_buf) }
        }

        fun read(error_buf: ByteBuffer): {{ e|type_name }} {
            {% if e.is_flat() %}
                return when(error_buf.getInt()) {
                {%- for variant in e.variants() %}
                {{ loop.index }} -> {{ e|type_name }}.{{ variant.name()|exception_name }}(String.read(error_buf))
                {%- endfor %}
                else -> throw RuntimeException("invalid error enum value, something is very wrong!!")
            }
            {% else %}

            return when(error_buf.getInt()) {
                {%- for variant in e.variants() %}
                {{ loop.index }} -> {{ e|type_name }}.{{ variant.name()|exception_name }}({% if variant.has_fields() %}
                    {% for field in variant.fields() -%}
                    {{ "error_buf"|read_var(field) }}{% if loop.last %}{% else %},{% endif %}
                    {% endfor -%}
                {%- endif -%})
                {%- endfor %}
                else -> throw RuntimeException("invalid error enum value, something is very wrong!!")
            }
            {%- endif %}
        }
    }

    {% if self.contains_object_references() %}
    @Suppress("UNNECESSARY_SAFE_CALL") // codegen is much simpler if we unconditionally emit safe calls here
    override fun destroy() {
        when(this) {
            {%- for variant in e.variants() %}
            is {{ e|type_name }}.{{ variant.name()|class_name }} -> {
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
