{% match func.documentation() -%}
  {% when Some with (docs) %}
    """
    {{ docs.description }}

    {%- if docs.arguments_descriptions.len() > 0 %}
    
    Parameters:

    {% for arg in func.arguments() -%}
    - `{{ arg.name() }}`: description
    {% endfor %} 
    {% endif -%}

    {%- if docs.return_description.is_some() %}

    Returns: description
    {% endif %}
    """
  {% when None %}
{%- endmatch %}
