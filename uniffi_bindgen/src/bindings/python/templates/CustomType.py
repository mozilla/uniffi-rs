{%- match config %}
{%- when None %}
{#- No custom type config, just forward all methods to our builtin type #}
class FfiConverterType{{ name }}:
    @staticmethod
    def write(value, buf):
        {{ "value"|write_var("buf", builtin) }}

    @staticmethod
    def read(buf):
        return {{ "buf"|read_var(builtin) }}

    @staticmethod
    def lift(value):
        return {{ "value"|lift_var(builtin) }}

    @staticmethod
    def lower(value):
        return {{ "value"|lower_var(builtin) }}
{%- when Some with (config) %}
{#- Custom type config supplied, use it to convert the builtin type #}
class FfiConverterType{{ name }}:
    @staticmethod
    def write(value, buf):
        builtin_value = {{ config.from_custom.render("value") }}
        {{ "builtin_value"|write_var("buf", builtin) }}

    @staticmethod
    def read(buf):
        builtin_value = {{ "buf"|read_var(builtin) }}
        return {{ config.into_custom.render("builtin_value") }}

    @staticmethod
    def lift(value):
        builtin_value = {{ "value"|lift_var(builtin) }}
        return {{ config.into_custom.render("builtin_value") }}

    @staticmethod
    def lower(value):
        builtin_value = {{ config.from_custom.render("value") }}
        return {{ "builtin_value"|lower_var(builtin) }}
{%- endmatch %}
