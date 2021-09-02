{% import "macros.kt" as kt %}

// Error {{ e.name() }}
{% if e.is_flat() %}
sealed class {{ e.nm() }}(message: String): Exception(message){% if contains_object_references %}, Disposable {% endif %} {
        // Each variant is a nested class
        // Flat enums carries a string error message, so no special implementation is necessary.
        {% for variant in e.variants() -%}
        class {{ e.variant_name(variant) }}(message: String) : {{ e.nm() }}(message)
        {% endfor %}

{%- else %}
sealed class {{ e.nm() }}: Exception(){% if contains_object_references %}, Disposable {% endif %} {
    // Each variant is a nested class
    {% for variant in e.variants() -%}
    {% if !variant.has_fields() -%}
    class {{ e.variant_name(variant) }} : {{ e.nm() }}()
    {% else %}
    class {{ e.variant_name(variant) }}(
        {% for field in variant.fields() -%}
        val {{ field.nm() }}: {{ field.type_().nm() }}{% if loop.last %}{% else %}, {% endif %}
        {% endfor -%}
    ) : {{ e.nm() }}()
    {%- endif %}
    {% endfor %}

{%- endif %}

    companion object ErrorHandler : CallStatusErrorHandler<{{ e.nm() }}> {
        override fun lift(error_buf: RustBuffer.ByValue): {{ e.nm() }} {
            return liftFromRustBuffer(error_buf) { error_buf -> read(error_buf) }
        }

        fun read(error_buf: ByteBuffer): {{ e.nm() }} {
            {% if e.is_flat() %}
                return when(error_buf.getInt()) {
                {%- for variant in e.variants() %}
                {{ loop.index }} -> {{ e.nm() }}.{{ e.variant_name(variant) }}(String.read(error_buf))
                {%- endfor %}
                else -> throw RuntimeException("invalid error enum value, something is very wrong!!")
            }
            {% else %}

            return when(error_buf.getInt()) {
                {%- for variant in e.variants() %}
                {{ loop.index }} -> {{ e.nm() }}.{{ e.variant_name(variant) }}({% if variant.has_fields() %}
                    {% for field in variant.fields() -%}
                    {{ field.type_().read("error_buf") }}{% if loop.last %}{% else %},{% endif %}
                    {% endfor -%}
                {%- endif -%})
                {%- endfor %}
                else -> throw RuntimeException("invalid error enum value, something is very wrong!!")
            }
            {%- endif %}
        }
    }

    {% if contains_object_references %}
    @Suppress("UNNECESSARY_SAFE_CALL") // codegen is much simpler if we unconditionally emit safe calls here
    override fun destroy() {
        when(this) {
            {%- for variant in e.variants() %}
            is {{ e.nm() }}.{{ e.variant_name(variant) }} -> {
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
