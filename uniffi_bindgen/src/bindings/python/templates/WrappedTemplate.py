{%- let inner_type = self.inner() %}
{%- let outer_type = self.outer() %}
{%- let canonical_type_name = outer_type|canonical_name %}

class {{ canonical_type_name }}:
    @staticmethod
    def _write(value, buf):
        {{ "value"|write_var("buf", inner_type) }}

    @staticmethod
    def _read(buf):
        return {{ "buf"|read_var(inner_type) }}

    @staticmethod
    def _lift(buf):
        return {{ "buf"|lift_var(inner_type) }}

    @staticmethod
    def _lower(value):
        return {{ "value"|lower_var(inner_type) }}
