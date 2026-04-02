package {{ package.name }}

{%- for type_def in package.type_definitions %}
{%- match type_def %}
{%- when TypeDefinition::Record(rec) %}
{% include "Record.kt" %}
{%- when TypeDefinition::Enum(en) %}
{% include "Enum.kt" %}
{%- endmatch %}
{%- endfor %}

{%- for func in package.functions %}
{% include "Function.kt" %}
{%- endfor %}
