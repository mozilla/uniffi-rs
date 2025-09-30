@dataclass
class {{ rec.self_type.type_name }}:
    {{ rec.docstring|docstring(4) -}}
    {%- if !rec.fields.is_empty() -%}
    def __init__(self, *, {% for field in rec.fields %}
    {{- field.name }}: {{- field.ty.type_name}}
    {%- if let Some(default) = field.default %} = {{ default.arg_literal }}{% endif %}
    {%- if !loop.last %}, {% endif %}
    {%- endfor %}):
        {%- for field in rec.fields %}
        {%- match field.default %}
        {%- when None %}
        self.{{ field.name }} = {{ field.name }}
        {%- when Some(default) %}
        {%- if default.is_arg_literal %}
        self.{{ field.name }} = {{ field.name }}
        {%- else %}
        if {{ field.name }} is {{ default.arg_literal }}:
            self.{{ field.name }} = {{ default.py_default }}
        else:
            self.{{ field.name }} = {{ field.name }}
        {%- endif %}
        {%- endmatch %}
        {%- endfor %}
    {%- endif -%}

    {%- let uniffi_trait_methods = rec.uniffi_trait_methods -%}

    {% filter indent(4) %}
    {% include "UniffiTraitImpls.py" %}
    {% endfilter %}

    {# "builtin" methods not handled by a uniffi_trait #}
    {%- if uniffi_trait_methods.display_fmt.is_none() %}
    def __str__(self):
        return "{{ rec.self_type.type_name }}({% for field in rec.fields %}{{ field.name }}={}{% if loop.last %}{% else %}, {% endif %}{% endfor %})".format({% for field in rec.fields %}self.{{ field.name }}{% if loop.last %}{% else %}, {% endif %}{% endfor %})
    {%- endif %}

    {%- if uniffi_trait_methods.eq_eq.is_none() %}
    def __eq__(self, other):
        {%- for field in rec.fields %}
        if self.{{ field.name }} != other.{{ field.name }}:
            return False
        {%- endfor %}
        return True
    {%- endif %}

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
