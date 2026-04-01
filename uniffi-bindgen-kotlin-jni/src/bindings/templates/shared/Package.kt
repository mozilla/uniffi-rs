package uniffi

class InternalException(message: String) : Exception(message)

{% include "FfiBuffer.kt" %}
{% include "Scaffolding.kt" %}

{%- for package in root.packages %}
{%- for type_def in package.type_definitions %}
{%- match type_def %}
{%- when TypeDefinition::Record(rec) %}
{% include "RecordFfi.kt" %}
{%- endmatch %}
{%- endfor %}
{%- endfor %}
