
{% match func.documentation() -%}
  {% when Some with (docs) %}
/**
* {{ docs.description }}

    {%- if docs.arguments_descriptions.len() > 0 %}
*
    {% for arg in func.arguments() -%}
* @param[{{ arg.name() }}] description.
    {% endfor -%} 
    {% endif -%}

    {%- if docs.return_description.is_some() %}
*
* @return something something.
    {% endif %}
*/
  {%- when None %}
{%- endmatch %}
