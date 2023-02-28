{% match variant.documentation() -%}
{% when Some with (docs) %}"""
{% for line in docs.lines() %}    {{ line }} 
{% endfor %}    """
{%- when None %}
{%- endmatch %}