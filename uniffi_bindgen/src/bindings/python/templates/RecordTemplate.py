{% import "macros.py" as py %}
{%- let rec = self.inner() %}
class {{ rec|type_name }}(ViaFfiUsingByteBuffer, object):
    def __init__(self, {% call py::field_list_decl(rec) %}):
        {%- for field in rec.fields() %}
        self.{{ field.name()|var_name }} = {{ field.name()|var_name }}
        {%- endfor %}

    def __str__(self):
        return "{{ rec|type_name }}({% for field in rec.fields() %}{{ field.name() }}={}{% if loop.last %}{% else %}, {% endif %}{% endfor %})".format({% for field in rec.fields() %}self.{{ field.name() }}{% if loop.last %}{% else %}, {% endif %}{% endfor %})

    def __eq__(self, other):
        {%- for field in rec.fields() %}
        if self.{{ field.name()|var_name }} != other.{{ field.name()|var_name }}:
            return False
        {%- endfor %}
        return True

    @staticmethod
    def _read(buf):
        return {{ rec|type_name }}(
            {%- for field in rec.fields() %}
            {{ field.name()|var_name }}={{ "buf"|read_var(field.type_()) }}{% if loop.last %}{% else %},{% endif %}
            {%- endfor %}
        )

    def _write(self, buf):
        {%- for field in rec.fields() %}
        {{ "self.{}"|format(field.name())|write_var("buf", field.type_()) }}
        {%- endfor %}
