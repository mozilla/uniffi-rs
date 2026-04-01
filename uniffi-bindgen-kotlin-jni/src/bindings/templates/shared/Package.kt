package uniffi

class InternalException(message: kotlin.String) : Exception(message)

{% include "LiftLower.kt" %}
{% include "Scaffolding.kt" %}

{%- for type_def in root.ffi_type_definitions() %}
{%- match type_def %}
{%- when TypeDefinition::Record(rec) %}
{% include "RecordFfi.kt" %}
{%- endmatch %}
{%- endfor %}
