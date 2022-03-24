{%- match config %}
{%- when None %}
{#- Define the type using typealiases to the builtin #}
public typealias {{ name }} = {{ builtin|type_name }}
public typealias {{ outer|ffi_converter_name }} = {{ builtin|ffi_converter_name }}

{%- when Some with (config) %}

{%- let ffi_type_name=builtin.ffi_type().borrow()|ffi_type_name %}

{# When the config specifies a different type name, create a typealias for it #}
{%- match config.type_name %}
{%- when Some(concrete_type_name) %}
public typealias {{ name }} = {{ concrete_type_name }}
{%- else %}
{%- endmatch %}

public object {{ outer|ffi_converter_name }}: FfiConverter<{{ name }}, {{ ffi_type_name }}> {
    override fun lift(value: {{ ffi_type_name }}): {{ name }} {
        val builtinValue = {{ builtin|lift_fn }}(value)
        return {{ config.into_custom.render("builtinValue") }}
    }

    override fun lower(value: {{ name }}): {{ ffi_type_name }} {
        val builtinValue = {{ config.from_custom.render("value") }}
        return {{ builtin|lower_fn }}(builtinValue)
    }

    override fun read(buf: ByteBuffer): {{ name }} {
        val builtinValue = {{ builtin|read_fn }}(buf)
        return {{ config.into_custom.render("builtinValue") }}
    }

    override fun allocationSize(value: {{ name }}): Int {
        val builtinValue = {{ config.from_custom.render("value") }}
        return {{ builtin|allocation_size_fn }}(builtinValue)
    }

    override fun write(value: {{ name }}, buf: ByteBuffer) {
        val builtinValue = {{ config.from_custom.render("value") }}
        {{ builtin|write_fn }}(builtinValue, buf)
    }
}
{%- endmatch %}
