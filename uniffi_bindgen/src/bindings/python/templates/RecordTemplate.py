{%- let rec = ci.get_record_definition(name).unwrap() %}
class {{ type_name }}:
    {%- call py::docstring(rec, 4) %}
    {%- for field in rec.fields() %}
    {{ field.name() }}: "{{ field|type_name }}"
    {%- call py::docstring(field, 4) %}
    {%- endfor %}

    {%- if rec.has_fields() %}
    def __init__(self, *, {% for field in rec.fields() %}
    {{- field.name() }}: "{{- field|type_name }}"
    {%- if field.default_value().is_some() %} = _DEFAULT{% endif %}
    {%- if !loop.last %}, {% endif %}
    {%- endfor %}):
        {%- for field in rec.fields() %}
        {%- let field_name = field.name() %}
        {%- match field.default_value() %}
        {%- when None %}
        self.{{ field_name }} = {{ field_name }}
        {%- when Some(literal) %}
        if {{ field_name }} is _DEFAULT:
            self.{{ field_name }} = {{ literal|literal_py(field) }}
        else:
            self.{{ field_name }} = {{ field_name }}
        {%- endmatch %}
        {%- endfor %}
    {%- endif %}

    def __str__(self):
        return "{{ type_name }}({% for field in rec.fields() %}{{ field.name() }}={}{% if loop.last %}{% else %}, {% endif %}{% endfor %})".format({% for field in rec.fields() %}self.{{ field.name() }}{% if loop.last %}{% else %}, {% endif %}{% endfor %})

    def __eq__(self, other):
        {%- for field in rec.fields() %}
        if self.{{ field.name() }} != other.{{ field.name() }}:
            return False
        {%- endfor %}
        return True

class {{ ffi_converter_name }}(_UniffiConverterRustBuffer):
    @staticmethod
    def read(buf):
        return {{ type_name }}(
            {%- for field in rec.fields() %}
            {{ field.name() }}={{ field|read_fn }}(buf),
            {%- endfor %}
        )

    @staticmethod
    def check_lower(value):
        {%- if rec.fields().is_empty() %}
        pass
        {%- else %}
        {%- for field in rec.fields() %}
        {{ field|check_lower_fn }}(value.{{ field.name() }})
        {%- endfor %}
        {%- endif %}

    @staticmethod
    def write(value, buf):
        {%- if rec.has_fields() %}
        {%- for field in rec.fields() %}
        {{ field|write_fn }}(value.{{ field.name() }}, buf)
        {%- endfor %}
        {%- else %}
        pass
        {%- endif %}
