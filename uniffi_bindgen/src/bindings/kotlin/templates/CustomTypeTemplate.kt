{%- match config %}
{%- when None %}
{#- No custom type config, just forward all methods to our builtin type #}
internal typealias {{ ffi_converter_name }} = {{ builtin|ffi_converter_name }}

{%- when Some with (config) %}
{#- Custom type config supplied, use it to convert the builtin type #}

{%- call kt::add_optional_imports(config.imports) %}

object {{ ffi_converter_name }} {
    fun write(value: {{ type_name }}, buf: RustBufferBuilder) {
        val builtinValue = {{ config.from_custom.render("value") }}
        {{ builtin|write_fn }}(builtinValue, buf)
    }

    fun read(buf: ByteBuffer): {{ type_name }} {
        val builtinValue = {{ builtin|read_fn }}(buf)
        return {{ config.into_custom.render("builtinValue") }}
    }

    fun lift(value: {{ builtin.ffi_type()|ffi_type_name }}): {{ type_name }} {
        val builtinValue = {{ builtin|lift_fn }}(value)
        return {{ config.into_custom.render("builtinValue") }}
    }

    fun lower(value: {{ type_name }}): {{ builtin.ffi_type()|ffi_type_name }} {
        val builtinValue = {{ config.from_custom.render("value") }}
        return {{ builtin|lower_fn }}(builtinValue)
    }
}
{%- endmatch %}
