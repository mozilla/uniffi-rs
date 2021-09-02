{#
// Kotlin's `enum class` constuct doesn't support variants with associated data,
// but is a little nicer for consumers than its `sealed class` enum pattern.
// So, we switch here, using `enum class` for enums with no associated data
// and `sealed class` for the general case.
#}
{% import "macros.kt" as kt %}
{%- if e.is_flat() %}

enum class {{ e.nm() }} {
    {% for variant in e.variants() -%}
    {{ e.variant_name(variant) }}{% if loop.last %};{% else %},{% endif %}
    {%- endfor %}

    companion object {
        internal fun lift(rbuf: RustBuffer.ByValue): {{ e.nm() }} {
            return liftFromRustBuffer(rbuf) { buf -> {{ e.nm() }}.read(buf) }
        }

        internal fun read(buf: ByteBuffer) =
            try { values()[buf.getInt() - 1] }
            catch (e: IndexOutOfBoundsException) {
                throw RuntimeException("invalid enum value, something is very wrong!!", e)
            }
    }

    internal fun lower(): RustBuffer.ByValue {
        return lowerIntoRustBuffer(this, {v, buf -> v.write(buf)})
    }

    internal fun write(buf: RustBufferBuilder) {
        buf.putInt(this.ordinal + 1)
    }
}

{% else %}

sealed class {{ e.nm() }}{% if contains_object_references %}: Disposable {% endif %} {
    {% for variant in e.variants() -%}
    {% if !variant.has_fields() -%}
    object {{ e.variant_name(variant) }} : {{ e.nm() }}()
    {% else -%}
    data class {{ e.variant_name(variant) }}(
        {% for field in variant.fields() -%}
        val {{ field.nm() }}: {{ field.type_().nm() }}{% if loop.last %}{% else %}, {% endif %}
        {% endfor -%}
    ) : {{ e.nm() }}()
    {%- endif %}
    {% endfor %}

    companion object {
        internal fun lift(rbuf: RustBuffer.ByValue): {{ e.nm() }} {
            return liftFromRustBuffer(rbuf) { buf -> {{ e.nm() }}.read(buf) }
        }

        internal fun read(buf: ByteBuffer): {{ e.nm() }} {
            return when(buf.getInt()) {
                {%- for variant in e.variants() %}
                {{ loop.index }} -> {{ e.nm() }}.{{ e.variant_name(variant) }}{% if variant.has_fields() %}(
                    {% for field in variant.fields() -%}
                    {{ field.type_().read("buf") }}{% if loop.last %}{% else %},{% endif %}
                    {% endfor -%}
                ){%- endif -%}
                {%- endfor %}
                else -> throw RuntimeException("invalid enum value, something is very wrong!!")
            }
        }
    }

    internal fun lower(): RustBuffer.ByValue {
        return lowerIntoRustBuffer(this, {v, buf -> v.write(buf)})
    }

    internal fun write(buf: RustBufferBuilder) {
        when(this) {
            {%- for variant in e.variants() %}
            is {{ e.nm() }}.{{ e.variant_name(variant) }} -> {
                buf.putInt({{ loop.index }})
                {% for field in variant.fields() -%}
                {{ field.type_().write(field.nm(), "buf") }}
                {% endfor %}
            }
            {%- endfor %}
        }.let { /* this makes the `when` an expression, which ensures it is exhaustive */ }
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

{% endif %}
