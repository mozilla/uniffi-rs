
{% match func.documentation() -%}
  {% when Some with (docs) %}
  # {{ docs.description }}

    {%- if docs.arguments_descriptions.len() > 0 %}
  # 
    {% for arg in func.arguments() -%}
  # @param {{ arg.name() }} [ArgType] description
    {% endfor %} 
    {% endif -%}

    {%- if docs.return_description.is_some() %}
  # @return [FunctionReturnValue] return field description
    {% endif %}
  {%- when None %}
{%- endmatch %}
