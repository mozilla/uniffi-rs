{#
// Kotlin's `enum class` constuct doesn't support variants with associated data,
// but is a little nicer for consumers than its `sealed class` enum pattern.
// So, we switch here, using `enum class` for enums with no associated data
// and `sealed class` for the general case.
#}
{% import "macros.kt" as kt %}
{%- let e = self.inner() %}
{%- if e.is_flat() %}

enum class {{ e.name()|class_name }} {
    {% for variant in e.variants() -%}
    {{ variant.name()|enum_variant }}{% if loop.last %};{% else %},{% endif %}
    {%- endfor %}

    companion object {
        internal fun lift(rbuf: RustBuffer.ByValue): {{ e.name()|class_name }} {
            return liftFromRustBuffer(rbuf) { buf -> {{ e.name()|class_name }}.read(buf) }
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

sealed class {{ e.name()|class_name }}{% if self.contains_object_references() %}: Disposable {% endif %} {
    {% for variant in e.variants() -%}
    {% if !variant.has_fields() -%}
    object {{ variant.name()|class_name }} : {{ e.name()|class_name }}()
    {% else -%}
    data class {{ variant.name()|class_name }}(
        {% for field in variant.fields() -%}
        val {{ field.name()|var_name }}: {{ field|type_name}}{% if loop.last %}{% else %}, {% endif %}
        {% endfor -%}
    ) : {{ e.name()|class_name }}()
    {%- endif %}
    {% endfor %}

    companion object {
        internal fun lift(rbuf: RustBuffer.ByValue): {{ e.name()|class_name }} {
            return liftFromRustBuffer(rbuf) { buf -> {{ e.name()|class_name }}.read(buf) }
        }

        internal fun read(buf: ByteBuffer): {{ e.name()|class_name }} {
            return when(buf.getInt()) {
                {%- for variant in e.variants() %}
                {{ loop.index }} -> {{ e.name()|class_name }}.{{ variant.name()|class_name }}{% if variant.has_fields() %}(
                    {% for field in variant.fields() -%}
                    {{ "buf"|read_var(field) }}{% if loop.last %}{% else %},{% endif %}
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
            is {{ e.name()|class_name }}.{{ variant.name()|class_name }} -> {
                buf.putInt({{ loop.index }})
                {% for field in variant.fields() -%}
                {{ "(this.{})"|format(field.name())|write_var("buf", field) }}
                {% endfor %}
            }
            {%- endfor %}
        }.let { /* this makes the `when` an expression, which ensures it is exhaustive */ }
    }

    {% if self.contains_object_references() %}
    @Suppress("UNNECESSARY_SAFE_CALL") // codegen is much simpler if we unconditionally emit safe calls here
    override fun destroy() {
        when(this) {
            {%- for variant in e.variants() %}
            is {{ e.name()|class_name }}.{{ variant.name()|class_name }} -> {
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
