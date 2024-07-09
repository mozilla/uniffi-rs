# Record type {{ rec.name() }}
class {{ rec.name()|class_name_rb(config) }}
  attr_reader {% for field in rec.fields() %}:{{ field.name()|var_name_rb(config) }}{% if loop.last %}{% else %}, {% endif %}{%- endfor %}

  def initialize({% for field in rec.fields() %}{{ field.name()|var_name_rb(config) -}}:
        {%- match field.default_value() %}
        {%- when Some with(literal) %} {{ literal|literal_rb(config) }}
        {%- else %}
        {%- endmatch %}
  {%- if loop.last %}{% else %}, {% endif -%}{% endfor %})
    {%- for field in rec.fields() %}
    @{{ field.name()|var_name_rb(config) }} = {{ field.name()|var_name_rb(config) }}
    {%- endfor %}
  end

  def ==(other)
    {%- for field in rec.fields() %}
    if @{{ field.name()|var_name_rb(config) }} != other.{{ field.name()|var_name_rb(config) }}
      return false
    end
    {%- endfor %}

    true
  end
end
