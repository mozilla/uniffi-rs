package uniffi

class InternalException(message: String) : Exception(message)

{% include "FfiBuffer.kt" %}
{% include "Scaffolding.kt" %}
{% include "Interfaces.kt" %}
{% include "CallbackInterfaces.kt" %}

{%- for type_def in root.ffi_type_definitions() %}
{%- match type_def %}
{%- when TypeDefinition::Record(rec) %}
{% include "RecordFfi.kt" %}
{%- if rec.self_type.is_used_as_error %}
{%- let error_type = rec.self_type %}
{% include "ThrowErrorFun.kt" %}
{%- endif %}
{%- when TypeDefinition::Enum(en) %}
{% include "EnumFfi.kt" %}
{%- if en.self_type.is_used_as_error %}
{%- let error_type = en.self_type %}
{% include "ThrowErrorFun.kt" %}
{%- endif %}
{%- when TypeDefinition::Class(cls) %}
{% include "ClassFfi.kt" %}
{%- if cls.self_type.is_used_as_error %}
{%- let error_type = cls.self_type %}
{% include "ThrowErrorFun.kt" %}
{%- endif %}
{%- when TypeDefinition::CallbackInterface(cbi) %}
{% include "CallbackInterfaceFfi.kt" %}
{%- when TypeDefinition::Custom(custom) %}
{% include "CustomFfi.kt" %}
{%- when TypeDefinition::Optional(opt) %}
{% include "OptionalFfi.kt" %}
{%- when TypeDefinition::Sequence(seq) %}
{% include "SequenceFfi.kt" %}
{%- when TypeDefinition::Map(map) %}
{% include "MapFfi.kt" %}
{%- when TypeDefinition::Interface(_) %}
{%- endmatch %}
{%- endfor %}
