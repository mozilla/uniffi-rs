{%- let builtin = custom.builtin %}
{%- match custom.config %}
{% when None %}
{#- No custom type config, just forward all methods to our builtin type #}
class {{ custom|ffi_converter_name }}:
    @staticmethod
    def write(value, buf):
        {{ builtin|ffi_converter_name }}.write(value, buf)

    @staticmethod
    def read(buf):
        return {{ builtin|ffi_converter_name }}.read(buf)

    @staticmethod
    def lift(value):
        return {{ builtin|ffi_converter_name }}.lift(value)

    @staticmethod
    def check_lower(value):
        return {{ builtin|ffi_converter_name }}.check_lower(value)

    @staticmethod
    def lower(value):
        return {{ builtin|ffi_converter_name }}.lower(value)

{# Render a type alias from the custom type name to the concrete type name #}
{{ custom.name }} = {{ builtin|type_name -}}

{%- when Some(config) %}

{%- if let Some(type_name) = config.type_name %}
{# Render a type alias from the custom type name to the concrete type name #}
{{ custom.name }} = {{ type_name }}
{%- endif %}

{#- Custom type config supplied, use it to convert the builtin type #}
class {{ custom|ffi_converter_name }}:
    @staticmethod
    def write(value, buf):
        builtin_value = {{ config.lower("value") }}
        {{ builtin|write_fn }}(builtin_value, buf)

    @staticmethod
    def read(buf):
        builtin_value = {{ builtin|read_fn }}(buf)
        return {{ config.lift("builtin_value") }}

    @staticmethod
    def lift(value):
        builtin_value = {{ builtin|lift_fn }}(value)
        return {{ config.lift("builtin_value") }}

    @staticmethod
    def check_lower(value):
        builtin_value = {{ config.lower("value") }}
        return {{ builtin|check_lower_fn }}(builtin_value)

    @staticmethod
    def lower(value):
        builtin_value = {{ config.lower("value") }}
        return {{ builtin|lower_fn }}(builtin_value)
{%- endmatch %}
