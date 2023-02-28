# Record type {{ rec.name() }}
{% include "RecordDocsTemplate.rb" -%}
class {{ rec.name()|class_name_rb }}
  {% for field in rec.fields() %}{% include "AttributeDocTemplate.rb" %}attr_reader :{{ field.name()|var_name_rb }}
  
  {% endfor %}

  def initialize({% for field in rec.fields() %}{{ field.name()|var_name_rb }}{% if loop.last %}{% else %}, {% endif %}{% endfor %})
    {%- for field in rec.fields() %}
    @{{ field.name()|var_name_rb }} = {{ field.name()|var_name_rb }}
    {%- endfor %}
  end

  def ==(other)
    {%- for field in rec.fields() %}
    if @{{ field.name()|var_name_rb }} != other.{{ field.name()|var_name_rb }}
      return false
    end
    {%- endfor %}

    true
  end
end
