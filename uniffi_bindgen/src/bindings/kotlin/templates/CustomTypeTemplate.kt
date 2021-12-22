{%- match config %}
{%- when None %}
{#- No custom type config, just forward all methods to our builtin type #}
internal typealias {{ outer|ffi_converter_name }} = {{ builtin|ffi_converter_name }}

{%- when Some with (config) %}
object {{ outer|ffi_converter_name }} {
    {#- Custom type config supplied, use it to convert the builtin type #}
    fun write(value: {{ self.type_name(config) }}, buf: RustBufferBuilder) {
        val builtinValue = {{ config.from_custom.render("value") }}
        {{ builtin|write_fn }}(builtinValue, buf)
    }

    fun read(buf: ByteBuffer): {{ self.type_name(config) }} {
        val builtinValue = {{ builtin|read_fn }}(buf)
        return {{ config.into_custom.render("builtinValue") }}
    }

    fun lift(value: {{ self.builtin_ffi_type()|ffi_type_name }}): {{ self.type_name(config) }} {
        val builtinValue = {{ builtin|lift_fn }}(value)
        return {{ config.into_custom.render("builtinValue") }}
    }

    fun lower(value: {{ self.type_name(config) }}): {{ self.builtin_ffi_type()|ffi_type_name }} {
        val builtinValue = {{ config.from_custom.render("value") }}
        return {{ builtin|lower_fn }}(builtinValue)
    }
}
{%- endmatch %}
