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
        override fun lift(errorBuf: RustBuffer.ByValue): {{ e.nm() }} {
            return {{ e.lift() }}(errorBuf)
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

object {{ e.ffi_converter_name() }}: FFIConverterRustBuffer<{{ e.nm() }}> {
    override fun read(buf: ByteBuffer): {{ e.nm() }} {
        return when(buf.getInt()) {
            {%- for variant in e.variants() %}
            {{ loop.index }} -> {{ e.nm() }}.{{ e.variant_name(variant) }}(
                {%- if e.is_flat() %}
                {{ Type::String.read() }}(buf),
                {%- else %}
                {%- for field in variant.fields() -%}
                {{ field.type_().read() }}(buf),
                {%- endfor %}
                {%- endif %}
            )
            {%- endfor %}
            else -> throw RuntimeException("invalid enum value, something is very wrong!!")
        }
    }

    override fun write(v: {{ e.nm() }}, buf: RustBufferBuilder) {
        when(v) {
            {%- for variant in e.variants() %}
            is {{ e.nm() }}.{{ e.variant_name(variant) }} -> {
                buf.putInt({{ loop.index }})
                {% for field in variant.fields() -%}
                {{ field.type_().write() }}(v.{{ field.nm() }}, buf)
                {% endfor %}
            }
            {%- endfor %}
        }.let { /* this makes the `when` an expression, which ensures it is exhaustive */ }
    }
}
