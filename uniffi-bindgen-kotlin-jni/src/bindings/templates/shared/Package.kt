@file:OptIn(kotlin.ExperimentalUnsignedTypes::class)
package uniffi

class InternalException(message: kotlin.String) : Exception(message)

{% include "FfiBuffer.kt" %}
{% include "LiftLower.kt" %}
{% include "Scaffolding.kt" %}
{% include "Interfaces.kt" %}

{%- for type_def in root.ffi_type_definitions() %}
{%- match type_def %}
{%- when TypeDefinition::Record(rec) %}
{% include "RecordFfi.kt" %}
{%- when TypeDefinition::Enum(en) %}
{% include "EnumFfi.kt" %}
{%- when TypeDefinition::Class(cls) %}
{% include "ClassFfi.kt" %}
{%- when TypeDefinition::Custom(custom) %}
{% include "CustomFfi.kt" %}
{%- when TypeDefinition::Optional(opt) %}
{% include "OptionalFfi.kt" %}
{%- when TypeDefinition::Sequence(seq) %}
{% include "SequenceFfi.kt" %}
{%- when TypeDefinition::Map(map) %}
{% include "MapFfi.kt" %}
{%- when TypeDefinition::Set(set) %}
{% include "SetFfi.kt" %}
{%- when TypeDefinition::Interface(_) %}
{%- endmatch %}
{%- endfor %}
