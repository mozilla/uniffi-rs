
{% match func.documentation() -%}
  {% when Some with (docs) %}
{% for line in docs.description.lines() %}# {{ line }}
{% endfor -%}

    {%- if docs.arguments_descriptions.len() > 0 %}# 
    {% for arg in func.arguments() -%}# @param [{{ canonical_name(arg.type_()) }}] {{ arg.name() }} {{ docs.arguments_descriptions[arg.name()] }}
    {% endfor -%} 
    {% endif -%}

    {%- match docs.return_description -%}
      {% when Some with (desc) %}# @return [{{ canonical_name(func.return_type().unwrap()) }}] {{ desc }}
      {%- when None %}
    {%- endmatch %}
  {%- when None %}
{%- endmatch -%}