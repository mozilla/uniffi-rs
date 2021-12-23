{% import "macros.kt" as kt %}
{%- for type_ in ci.iter_types() %}
{%- include "Type.kt" %}
{%- endfor %}

{%- for func in ci.iter_function_definitions() %}
{%- include "TopLevelFunctionTemplate.kt" %}
{%- endfor %}
