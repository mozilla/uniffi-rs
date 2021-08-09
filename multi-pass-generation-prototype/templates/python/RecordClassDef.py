class {{ name }}:
    def __init__(self{% for field in fields %}, {{ field.0 }}{% endfor %}):
        {%- for field in fields %}
        self.{{ field.0 }} = {{ field.0 }}
        {%- endfor %}

