{#
// Kotlin's `enum class` constuct doesn't support variants with associated data,
// but is a little nicer for consumers than its `sealed class` enum pattern.
// So, we switch here, using `enum class` for enums with no associated data
// and `sealed class` for the general case.
#}
{% import "macros.kt" as kt %}
{%- let e = self.inner() %}
{%- let enum_class = e.name()|class_name_kt %}
{%- let type_ = e.type_() %}
{%- if e.is_flat() %}

enum class {{ enum_class }} {
    {% for variant in e.variants() -%}
    {{ variant.name()|enum_variant_kt }}{% if loop.last %};{% else %},{% endif %}
    {%- endfor %}
}

{% call kt::unsigned_types_annotation(self) %}
object {{ type_|ffi_converter_name }} {
    internal fun read(buf: ByteBuffer) = try {
        {{ enum_class }}.values()[buf.getInt() - 1]
    } catch (e: IndexOutOfBoundsException) {
        throw RuntimeException("invalid enum value, something is very wrong!!", e)
    }

    internal fun write(e: {{ enum_class }}, buf: RustBufferBuilder) = buf.putInt(e.ordinal + 1)

    {% call kt::lift_and_lower_from_read_and_write(type_) %}
}

{% else %}

{% call kt::unsigned_types_annotation(self) %}
sealed class {{ enum_class }}{% if self.contains_object_references() %}: Disposable {% endif %} {
    {% for variant in e.variants() -%}
    {% if !variant.has_fields() -%}
    object {{ variant.name()|class_name_kt }} : {{ enum_class }}()
    {% else -%}
    data class {{ variant.name()|class_name_kt }}(
        {% for field in variant.fields() -%}
        val {{ field.name()|var_name_kt }}: {{ field.type_()|type_kt}}{% if loop.last %}{% else %}, {% endif %}
        {% endfor -%}
    ) : {{ enum_class }}()
    {%- endif %}
    {% endfor %}

    {% if self.contains_object_references() %}
    @Suppress("UNNECESSARY_SAFE_CALL") // codegen is much simpler if we unconditionally emit safe calls here
    override fun destroy() {
        when(this) {
            {%- for variant in e.variants() %}
            is {{ enum_class }}.{{ variant.name()|class_name_kt }} -> {
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

{% call kt::unsigned_types_annotation(self) %}
object {{ type_|ffi_converter_name }} {
    internal fun read(buf: ByteBuffer): {{ enum_class }} {
        return when(buf.getInt()) {
            {%- for variant in e.variants() %}
            {{ loop.index }} -> {{ enum_class }}.{{ variant.name()|class_name_kt }}{% if variant.has_fields() %}(
                {% for field in variant.fields() -%}
                {{ field.type_()|ffi_converter_name }}.read(buf){% if loop.last %}{% else %},{% endif %}
                {% endfor -%}
            ){%- endif -%}
            {%- endfor %}
            else -> throw RuntimeException("invalid enum value, something is very wrong!!")
        }
    }

    internal fun write(e: {{ enum_class }}, buf: RustBufferBuilder) {
        when(e) {
            {%- for variant in e.variants() %}
            is {{ enum_class }}.{{ variant.name()|class_name_kt }} -> {
                buf.putInt({{ loop.index }})
                {% for field in variant.fields() -%}
                {{ field.type_()|ffi_converter_name }}.write(e.{{ field.name()|var_name_kt }}, buf)
                {% endfor %}
            }
            {%- endfor %}
        }.let { /* this makes the `when` an expression, which ensures it is exhaustive */ }
    }

    {% call kt::lift_and_lower_from_read_and_write(type_) %}
}

{% endif %}
