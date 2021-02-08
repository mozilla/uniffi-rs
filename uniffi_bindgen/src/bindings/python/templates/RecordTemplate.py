class {{ rec.name()|class_name_py }}(object):
    def __init__(self,{% for field in rec.fields() %}{{ field.name()|var_name_py }}{% if loop.last %}{% else %}, {% endif %}{% endfor %}):
        {%- for field in rec.fields() %}
        self.{{ field.name()|var_name_py }} = {{ field.name()|var_name_py }}
        {%- endfor %}

    def __str__(self):
        return "{{ rec.name()|class_name_py }}({% for field in rec.fields() %}{{ field.name() }}={}{% if loop.last %}{% else %}, {% endif %}{% endfor %})".format({% for field in rec.fields() %}self.{{ field.name() }}{% if loop.last %}{% else %}, {% endif %}{% endfor %})

    def __eq__(self, other):
        {%- for field in rec.fields() %}
        if self.{{ field.name()|var_name_py }} != other.{{ field.name()|var_name_py }}:
            return False
        {%- endfor %}
        return True
