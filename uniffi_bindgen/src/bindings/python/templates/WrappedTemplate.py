{%- let inner_type = self.inner() %}
{%- let outer_type = self.outer() %}
{%- let canonical_type_name = outer_type|canonical_name %}

class {{ canonical_type_name }}:
    @staticmethod
    def _write(value, buf):
        {{ "value"|write_py("buf", inner_type) }}

    @staticmethod
    def _read(buf):
        return {{ "buf"|read_py(inner_type) }}

    @staticmethod
    def _lift(buf):
        return {{ "buf"|lift_py(inner_type) }}

    @staticmethod
    def _lower(value):
        return {{ "value"|lower_py(inner_type) }}
