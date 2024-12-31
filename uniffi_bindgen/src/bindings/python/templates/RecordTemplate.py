class {{ rec.name }}:
    {{ rec.docstring|docindent(4) -}}
    {%- for field in rec.fields %}
    {{ field.name }}: "{{ field.ty.type_name }}"
    {{ field.docstring|docindent(4) -}}
    {%- endfor %}

    {%- if rec.has_fields() %}
    def __init__(self, *, {% for field in rec.fields %}
    {{- field.name }}: "{{- field.ty.type_name }}"
    {%- if field.has_default() %} = _DEFAULT{% endif %}
    {%- if !loop.last %}, {% endif %}
    {%- endfor %}):
        {%- for field in rec.fields %}
        {%- let field_name = field.name %}
        {%- match field.default %}
        {%- when None %}
        self.{{ field_name }} = {{ field_name }}
        {%- when Some(literal) %}
        if {{ field_name }} is _DEFAULT:
            self.{{ field_name }} = {{ literal }}
        else:
            self.{{ field_name }} = {{ field_name }}
        {%- endmatch %}
        {%- endfor %}
    {%- endif %}

    def __str__(self):
        return "{{ rec.name }}({% for field in rec.fields %}{{ field.name }}={}{% if loop.last %}{% else %}, {% endif %}{% endfor %})".format({% for field in rec.fields %}self.{{ field.name }}{% if loop.last %}{% else %}, {% endif %}{% endfor %})

    def __eq__(self, other):
        {%- for field in rec.fields %}
        if self.{{ field.name }} != other.{{ field.name }}:
            return False
        {%- endfor %}
        return True

class {{ ffi_converter_name }}(_UniffiConverterRustBuffer):
    @staticmethod
    def read(buf):
        return {{ rec.name }}(
            {%- for field in rec.fields %}
            {{ field.name }}={{ field|read_fn }}(buf),
            {%- endfor %}
        )

    @staticmethod
    def check_lower(value):
        {%- for field in rec.fields %}
        {{ field|check_lower_fn }}(value.{{ field.name }})
        {%- else %}
        pass
        {%- endfor %}

    @staticmethod
    def write(value, buf):
        {%- for field in rec.fields %}
        {{ field|write_fn }}(value.{{ field.name }}, buf)
        {%- else %}
        pass
        {%- endfor %}
