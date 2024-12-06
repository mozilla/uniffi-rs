{%- for type_def in type_definitions %}

{%- let ffi_converter_name = type_def|ffi_converter_name %}

{% match type_def %}

{%- when TypeDefinition::Builtin(type_node) %}
{%- match type_node.kind %}
{%- when TypeKind::Boolean %}
{%- include "BooleanHelper.py" %}

{%- when TypeKind::Int8 %}
{%- include "Int8Helper.py" %}

{%- when TypeKind::Int16 %}
{%- include "Int16Helper.py" %}

{%- when TypeKind::Int32 %}
{%- include "Int32Helper.py" %}

{%- when TypeKind::Int64 %}
{%- include "Int64Helper.py" %}

{%- when TypeKind::UInt8 %}
{%- include "UInt8Helper.py" %}

{%- when TypeKind::UInt16 %}
{%- include "UInt16Helper.py" %}

{%- when TypeKind::UInt32 %}
{%- include "UInt32Helper.py" %}

{%- when TypeKind::UInt64 %}
{%- include "UInt64Helper.py" %}

{%- when TypeKind::Float32 %}
{%- include "Float32Helper.py" %}

{%- when TypeKind::Float64 %}
{%- include "Float64Helper.py" %}

{%- when TypeKind::String %}
{%- include "StringHelper.py" %}

{%- when TypeKind::Bytes %}
{%- include "BytesHelper.py" %}

{%- when TypeKind::Timestamp %}
{%- include "TimestampHelper.py" %}

{%- when TypeKind::Duration %}
{%- include "DurationHelper.py" %}


{%- when TypeKind::Optional { inner_type } %}
{%- include "OptionalTemplate.py" %}

{%- when TypeKind::Sequence { inner_type } %}
{%- include "SequenceTemplate.py" %}

{%- when TypeKind::Map { key_type, value_type } %}
{%- include "MapTemplate.py" %}

{%- else %}
# Invalid Builtin type: {type_def:?}")
{%- endmatch %}

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
