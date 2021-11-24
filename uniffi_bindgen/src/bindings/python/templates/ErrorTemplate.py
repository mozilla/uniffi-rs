{%- let e = self.inner() %}
class {{ e|type_name }}(ViaFfiUsingByteBuffer):

    {%- if e.is_flat() %}

    # Each variant is a nested class of the error itself.
    # It just carries a string error message, so no special implementation is necessary.
    {%- for variant in e.variants() %}
    class {{ variant.name()|class_name }}(ViaFfiUsingByteBuffer, Exception):
        def _write(self, buf):
            buf.writeI32({{ loop.index }})
            message = str(self)
            {{ "message"|write_var("buf", Type::String) }}
    {%- endfor %}

    @classmethod
    def _read(cls, buf):
        variant = buf.readI32()
        {% for variant in e.variants() -%}
        if variant == {{ loop.index }}:
            return cls.{{ variant.name()|class_name }}({{ "buf"|read_var(Type::String) }})
        {% endfor %}
        raise InternalError("Raw enum value doesn't match any cases")

    {%- else %}

    # Each variant is a nested class of the error itself.
    {%- for variant in e.variants() %}
    class {{ variant.name()|class_name }}(ViaFfiUsingByteBuffer, Exception):
        def __init__(self{% for field in variant.fields() %}, {{ field.name()|var_name }}{% endfor %}):
            {%- if variant.has_fields() %}
            {%- for field in variant.fields() %}
            self.{{ field.name()|var_name }} = {{ field.name()|var_name }}
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
            return "{{ e|type_name }}.{{ variant.name()|class_name }}({})".format(', '.join(field_parts))
            {%- else %}
            return "{{ e|type_name }}.{{ variant.name()|class_name }}"
            {%- endif %}

        def _write(self, buf):
            buf.writeI32({{ loop.index }})
            {%- for field in variant.fields() %}
            {{ "self.{}"|format(field.name()) |write_var("buf", field.type_()) }}
            {%- endfor %}
    {%- endfor %}

    @classmethod
    def _read(cls, buf):
        variant = buf.readI32()
        {% for variant in e.variants() -%}
        if variant == {{ loop.index }}:
            return cls.{{ variant.name()|class_name }}(
                {% for field in variant.fields() -%}
                {{ field.name()|var_name }}={{ "buf"|read_var(field.type_()) }}{% if loop.last %}{% else %},{% endif %}
                {% endfor -%}
            )
        {% endfor %}
        raise InternalError("Raw enum value doesn't match any cases")

    {%- endif %}
