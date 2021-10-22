{%- let e = self.inner() %}
class {{ e.name()|class_name_py }}(ViaFfiUsingByteBuffer):

    {%- if e.is_flat() %}

    # Each variant is a nested class of the error itself.
    # It just carries a string error message, so no special implementation is necessary.
    {%- for variant in e.variants() %}
    class {{ variant.name()|class_name_py }}(ViaFfiUsingByteBuffer, Exception):
        def _write(self, buf):
            buf.writeI32({{ loop.index }})
            message = str(self)
            {{ "message"|write_py("buf", Type::String) }}
    {%- endfor %}

    @classmethod
    def _read(cls, buf):
        variant = buf.readI32()
        {% for variant in e.variants() -%}
        if variant == {{ loop.index }}:
            return cls.{{ variant.name()|class_name_py }}({{ "buf"|read_py(Type::String) }})
        {% endfor %}
        raise InternalError("Raw enum value doesn't match any cases")

    {%- else %}

    # Each variant is a nested class of the error itself.
    {%- for variant in e.variants() %}
    class {{ variant.name()|class_name_py }}(ViaFfiUsingByteBuffer, Exception):
        def __init__(self{% for field in variant.fields() %}, {{ field.name()|var_name_py }}{% endfor %}):
            {%- if variant.has_fields() %}
            {%- for field in variant.fields() %}
            self.{{ field.name()|var_name_py }} = {{ field.name()|var_name_py }}
            {%- endfor %}
            {%- else %}
            pass
            {%- endif %}

        def __str__(self):
            {%- if variant.has_fields() %}
            field_parts = [
                {%- for field in variant.fields() %}
                '{{ field.name() }}={!r}'.format(self.{{ field.name() }}),
                {%- endfor %}
            ]
            return "{{ e.name()|class_name_py }}.{{ variant.name()|class_name_py }}({})".format(', '.join(field_parts))
            {%- else %}
            return "{{ e.name()|class_name_py }}.{{ variant.name()|class_name_py }}"
            {%- endif %}

        def _write(self, buf):
            buf.writeI32({{ loop.index }})
            {%- for field in variant.fields() %}
            {{ "self.{}"|format(field.name()) |write_py("buf", field.type_()) }}
            {%- endfor %}
    {%- endfor %}

    @classmethod
    def _read(cls, buf):
        variant = buf.readI32()
        {% for variant in e.variants() -%}
        if variant == {{ loop.index }}:
            return cls.{{ variant.name()|class_name_py }}(
                {% for field in variant.fields() -%}
                {{ field.name()|var_name_py }}={{ "buf"|read_py(field.type_()) }}{% if loop.last %}{% else %},{% endif %}
                {% endfor -%}
            )
        {% endfor %}
        raise InternalError("Raw enum value doesn't match any cases")

    {%- endif %}
