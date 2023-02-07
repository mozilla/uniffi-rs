
{% match func.documentation() -%}
  {% when Some with (docs) %}
/// {{ docs.description }}

    {%- if docs.arguments_descriptions.len() > 0 %}
/// 
/// - Parameters:
    {% for arg in func.arguments() -%}
///     - {{ arg.name() }}: argument description
    {% endfor -%} 
    {% endif -%}

    {%- if docs.return_description.is_some() %}
///
/// - Returns: The sloth's energy level after eating.
    {% endif %}
  {%- when None %}
{%- endmatch %}
