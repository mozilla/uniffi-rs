{% import "macros.py" as py %}
{%- let rec = self.inner() %}
class {{ rec|type_name }}:
    def __init__(self, {% for field in rec.fields() %}
    {{- field.name()|var_name }}
    {%- if field.default_value().is_some() %} = DEFAULT{% endif %}
    {%- if !loop.last %}, {% endif %}
    {%- endfor %}):
        {%- for field in rec.fields() %}
        {%- let field_name = field.name()|var_name %}
        {%- match field.default_value() %}
        {%- when None %}
        self.{{ field_name }} = {{ field_name }}
        {%- when Some with(literal) %}
        if {{ field_name }} is DEFAULT:
            self.{{ field_name }} = {{ literal|literal_py(field) }}
        else:
            self.{{ field_name }} = {{ field_name }}
        {%- endmatch %}
        {%- endfor %}

    def __str__(self):
        return "{{ rec|type_name }}({% for field in rec.fields() %}{{ field.name() }}={}{% if loop.last %}{% else %}, {% endif %}{% endfor %})".format({% for field in rec.fields() %}self.{{ field.name() }}{% if loop.last %}{% else %}, {% endif %}{% endfor %})

    def __eq__(self, other):
        {%- for field in rec.fields() %}
        if self.{{ field.name()|var_name }} != other.{{ field.name()|var_name }}:
            return False
        {%- endfor %}
        return True

class {{ rec|ffi_converter_name }}(FfiConverterRustBuffer):
    @staticmethod
    def read(buf):
        return {{ rec|type_name }}(
            {%- for field in rec.fields() %}
            {{ field.name()|var_name }}={{ field|read_fn }}(buf),
            {%- endfor %}
        )

    @staticmethod
    def write(value, buf):
        {%- for field in rec.fields() %}
        {{ field|write_fn }}(value.{{ field.name() }}, buf)
        {%- endfor %}
