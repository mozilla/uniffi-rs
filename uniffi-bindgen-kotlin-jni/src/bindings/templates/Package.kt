package {{ package.name }}

{%- for func in package.functions %}
{% include "Function.kt" %}
{%- endfor %}
