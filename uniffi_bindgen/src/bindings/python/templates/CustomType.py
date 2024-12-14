{%- match custom_type.config %}
{% when None %}
{#- No custom type config, just forward all methods to our builtin type #}
{{ custom_type.name }} = {{ custom_type.builtin.type_name }}

class {{ ffi_converter_name }}:
    @staticmethod
    def write(value, buf):
        {{ custom_type.builtin.ffi_converter_name }}.write(value, buf)

    @staticmethod
    def read(buf):
        return {{ custom_type.builtin.ffi_converter_name }}.read(buf)

    @staticmethod
    def lift(value):
        return {{ custom_type.builtin.ffi_converter_name }}.lift(value)

    @staticmethod
    def check_lower(value):
        return {{ custom_type.builtin.ffi_converter_name }}.check_lower(value)

    @staticmethod
    def lower(value):
        return {{ custom_type.builtin.ffi_converter_name }}.lower(value)

{%- when Some(config) %}
{%- if let Some(type_name) = config.type_name %}
{{ custom_type.name }} = {{ type_name }}
{%- endif %}
{#- Custom type config supplied, use it to convert the builtin type #}
class {{ ffi_converter_name }}:
    @staticmethod
    def write(value, buf):
        builtin_value = {{ config.from_custom.render("value") }}
        {{ custom_type.builtin|write_fn }}(builtin_value, buf)

    @staticmethod
    def read(buf):
        builtin_value = {{ custom_type.builtin|read_fn }}(buf)
        return {{ config.into_custom.render("builtin_value") }}

    @staticmethod
    def lift(value):
        builtin_value = {{ custom_type.builtin|lift_fn }}(value)
        return {{ config.into_custom.render("builtin_value") }}

    @staticmethod
    def check_lower(value):
        builtin_value = {{ config.from_custom.render("value") }}
        return {{ custom_type.builtin|check_lower_fn }}(builtin_value)

    @staticmethod
    def lower(value):
        builtin_value = {{ config.from_custom.render("value") }}
        return {{ custom_type.builtin|lower_fn }}(builtin_value)
{%- endmatch %}
