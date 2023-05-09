{%- let rec = ci.get_record_definition(name).unwrap() %}
class {{ type_name }}:

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
        return "{{ type_name }}({% for field in rec.fields() %}{{ field.name()|var_name }}={}{% if loop.last %}{% else %}, {% endif %}{% endfor %})".format({% for field in rec.fields() %}self.{{ field.name()|var_name }}{% if loop.last %}{% else %}, {% endif %}{% endfor %})

    def __eq__(self, other):
        {%- for field in rec.fields() %}
        if self.{{ field.name()|var_name }} != other.{{ field.name()|var_name }}:
            return False
        {%- endfor %}
        return True

    {%- if python_config.json_support %}

    def to_dict(self):
        return dict({
            {%- for field in rec.fields() %}
            {%- let field_name = field.name()|var_name %}
            {%- let field_from_type = field_name|from_type("self", field.type_().borrow(), ci, python_config) %}
            "{{ field_name }}": {{ field_from_type }}
            {%- if !loop.last %},{% endif %}
            {%- endfor %}
        })

    def to_json(self):
        return json.dumps(self.to_dict())


    @staticmethod
    def from_json(str):
        value = json.loads(str)
        return {{ type_name }}.from_dict(value)


    @staticmethod
    def from_dict(dict_value):
        {%- for field in rec.fields() %}
        {%- let field_name = field.name()|var_name %}
        {%- match field_name|into_type("dict_value", field.type_().borrow(), ci, python_config) %}
        {%- when None %}
        {%- when Some with(value) %}
        dict_value["{{ field_name }}"] = {{ value }}
        {%- endmatch %}
        {%- endfor %}

        {%- for field in rec.fields() %}
        {%- let field_name = field.name()|var_name %}
        {%- match field.default_value() %}
        {%- when None %}
        {{ field_name }} = dict_value.pop("{{ field_name }}", None)
        {%- when Some with(literal) %}
        {%- endmatch %}
        {%- endfor %}

        return {{ type_name }}({% for field in rec.fields() %}{% let field_name = field.name()|var_name %}{% match field.default_value() %}{% when None %}{{ field_name }}{% if !loop.last || rec.has_field_with_default()  %}, {% endif %}{%- when Some with(literal) %}{% endmatch %}{% endfor %}{% if rec.has_field_with_default() %}**dict_value{% endif %})
    {%- endif %}

class {{ ffi_converter_name }}(FfiConverterRustBuffer):
    @staticmethod
    def read(buf):
        return {{ type_name }}(
            {%- for field in rec.fields() %}
            {{ field.name()|var_name }}={{ field|read_fn }}(buf),
            {%- endfor %}
        )

    @staticmethod
    def write(value, buf):
        {%- for field in rec.fields() %}
        {{ field|write_fn }}(value.{{ field.name()|var_name }}, buf)
        {%- endfor %}
