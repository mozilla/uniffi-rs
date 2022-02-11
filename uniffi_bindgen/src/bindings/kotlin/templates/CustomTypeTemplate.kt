{%- match config %}
{%- when None %}
{#- No custom type config, just forward all methods to our builtin type #}
internal typealias {{ outer|ffi_converter_name }} = {{ builtin|ffi_converter_name }}

{%- when Some with (config) %}
{%- let type_name=self.type_name(config) %}
{%- let ffi_type_name=self.builtin_ffi_type()|ffi_type_name %}
object {{ outer|ffi_converter_name }}: FfiConverter<{{ type_name }}, {{ffi_type_name }}> {
    {#- Custom type config supplied, use it to convert the builtin type #}
    override fun lift(value: {{ ffi_type_name }}): {{ type_name }} {
        val builtinValue = {{ builtin|lift_fn }}(value)
        return {{ config.into_custom.render("builtinValue") }}
    }

    override fun lower(value: {{ type_name }}): {{ ffi_type_name }} {
        val builtinValue = {{ config.from_custom.render("value") }}
        return {{ builtin|lower_fn }}(builtinValue)
    }

    override fun read(buf: ByteBuffer): {{ type_name }} {
        val builtinValue = {{ builtin|read_fn }}(buf)
        return {{ config.into_custom.render("builtinValue") }}
    }

    override fun allocationSize(value: {{ type_name }}): Int {
        val builtinValue = {{ config.from_custom.render("value") }}
        return {{ builtin|allocation_size_fn }}(builtinValue)
    }

    override fun write(value: {{ type_name }}, buf: ByteBuffer) {
        val builtinValue = {{ config.from_custom.render("value") }}
        {{ builtin|write_fn }}(builtinValue, buf)
    }
}
{%- endmatch %}
