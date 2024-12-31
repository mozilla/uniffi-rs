{%- for type_def in type_definitions %}

{%- let ffi_converter_name = type_def.as_type().ffi_converter_name %}

{% match type_def %}

{%- when TypeDefinition::Simple(type_node) %}
{%- match type_node.kind %}
{%- when uniffi_meta::Type::Boolean %}
{%- include "BooleanHelper.py" %}

{%- when uniffi_meta::Type::Int8 %}
{%- include "Int8Helper.py" %}

{%- when uniffi_meta::Type::Int16 %}
{%- include "Int16Helper.py" %}

{%- when uniffi_meta::Type::Int32 %}
{%- include "Int32Helper.py" %}

{%- when uniffi_meta::Type::Int64 %}
{%- include "Int64Helper.py" %}

{%- when uniffi_meta::Type::UInt8 %}
{%- include "UInt8Helper.py" %}

{%- when uniffi_meta::Type::UInt16 %}
{%- include "UInt16Helper.py" %}

{%- when uniffi_meta::Type::UInt32 %}
{%- include "UInt32Helper.py" %}

{%- when uniffi_meta::Type::UInt64 %}
{%- include "UInt64Helper.py" %}

{%- when uniffi_meta::Type::Float32 %}
{%- include "Float32Helper.py" %}

{%- when uniffi_meta::Type::Float64 %}
{%- include "Float64Helper.py" %}

{%- when uniffi_meta::Type::String %}
{%- include "StringHelper.py" %}

{%- when uniffi_meta::Type::Bytes %}
{%- include "BytesHelper.py" %}

{%- when uniffi_meta::Type::Timestamp %}
{%- include "TimestampHelper.py" %}

{%- when uniffi_meta::Type::Duration %}
{%- include "DurationHelper.py" %}


{%- else %}
# Invalid Primitive type: {type_def:?}")
{%- endmatch %}

{%- when TypeDefinition::Optional(OptionalType { inner, .. }) %}
{%- include "OptionalTemplate.py" %}

{%- when TypeDefinition::Sequence(SequenceType { inner, .. }) %}
{%- include "SequenceTemplate.py" %}

{%- when TypeDefinition::Map(MapType { key, value, .. }) %}
{%- include "MapTemplate.py" %}


{%- when TypeDefinition::Enum(e) %}
{# For enums, there are either an error *or* an enum, they can't be both. #}

{%- if e.self_type.is_used_as_error %}
{%- include "ErrorTemplate.py" %}
{%- else %}
{%- include "EnumTemplate.py" %}
{% endif %}

{%- when TypeDefinition::Record(rec) %}
{%- include "RecordTemplate.py" %}

{%- when TypeDefinition::Interface(interface) %}
{%- include "ObjectTemplate.py" %}

{%- when TypeDefinition::CallbackInterface(cbi) %}
{%- include "CallbackInterfaceTemplate.py" %}

{%- when TypeDefinition::Custom(custom_type) %}
{%- include "CustomType.py" %}

{%- when TypeDefinition::External(_) %}
{%- endmatch %}

{% endfor %}
