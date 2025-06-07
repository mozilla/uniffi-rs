class {{ rec.self_type.type_name }}:
    {{ rec.docstring|docstring(4) -}}
    {% for field in rec.fields -%}
    {{ field.name }}: {{ field.ty.type_name}}
    {{ rec.docstring|docstring(4) -}}
    {% endfor -%}

    {%- if !rec.fields.is_empty() %}
    def __init__(self, *, {% for field in rec.fields %}
    {{- field.name }}: {{- field.ty.type_name}}
    {%- if field.default.is_some() %} = _DEFAULT{% endif %}
    {%- if !loop.last %}, {% endif %}
    {%- endfor %}):
        {%- for field in rec.fields %}
        {%- match field.default %}
        {%- when None %}
        self.{{ field.name }} = {{ field.name }}
        {%- when Some(lit) %}
        if {{ field.name }} is _DEFAULT:
            self.{{ field.name }} = {{ lit.py_lit }}
        else:
            self.{{ field.name }} = {{ field.name }}
        {%- endmatch %}
        {%- endfor %}
    {%- endif %}

    def __str__(self):
        return "{{ rec.self_type.type_name }}({% for field in rec.fields %}{{ field.name }}={}{% if loop.last %}{% else %}, {% endif %}{% endfor %})".format({% for field in rec.fields %}self.{{ field.name }}{% if loop.last %}{% else %}, {% endif %}{% endfor %})

    def __eq__(self, other):
        {%- for field in rec.fields %}
        if self.{{ field.name }} != other.{{ field.name }}:
            return False
        {%- endfor %}
        return True

class {{ rec.self_type.ffi_converter_name }}(_UniffiConverterRustBuffer):
    @staticmethod
    def read(buf):
        return {{ rec.self_type.type_name }}(
            {%- for field in rec.fields %}
            {{ field.name }}={{ field.ty.ffi_converter_name }}.read(buf),
            {%- endfor %}
        )

    @staticmethod
    def check_lower(value):
        {%- if rec.fields.is_empty() %}
        pass
        {%- else %}
        {%- for field in rec.fields %}
        {{ field.ty.ffi_converter_name }}.check_lower(value.{{ field.name }})
        {%- endfor %}
        {%- endif %}

    @staticmethod
    def write(value, buf):
        {%- if !rec.fields.is_empty() %}
        {%- for field in rec.fields %}
        {{ field.ty.ffi_converter_name }}.write(value.{{ field.name }}, buf)
        {%- endfor %}
        {%- else %}
        pass
        {%- endif %}
