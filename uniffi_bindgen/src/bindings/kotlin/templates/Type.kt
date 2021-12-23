{%- let type_name = type_|type_name %}
{%- let ffi_converter_name = type_|ffi_converter_name %}

{%- match type_ %}

{%- when Type::Boolean %}
{%- include "BooleanHelper.kt" %}

{%- when Type::String %}
{%- include "StringHelper.kt" %}

{%- when Type::Int8 %}
{%- include "Int8Helper.kt" %}

{%- when Type::Int16 %}
{%- include "Int16Helper.kt" %}

{%- when Type::Int32 %}
{%- include "Int32Helper.kt" %}

{%- when Type::Int64 %}
{%- include "Int64Helper.kt" %}

{%- when Type::UInt8 %}
{%- include "UInt8Helper.kt" %}

{%- when Type::UInt16 %}
{%- include "UInt16Helper.kt" %}

{%- when Type::UInt32 %}
{%- include "UInt32Helper.kt" %}

{%- when Type::UInt64 %}
{%- include "UInt64Helper.kt" %}

{%- when Type::Float32 %}
{%- include "Float32Helper.kt" %}

{%- when Type::Float64 %}
{%- include "Float64Helper.kt" %}

{%- when Type::Duration %}
{%- include "DurationHelper.kt" %}

{%- when Type::Timestamp %}
{%- include "TimestampHelper.kt" %}

{%- when Type::Enum with (name) %}
{%- let enum_ = ci.get_enum_definition(name).unwrap() %}
{%- let contains_object_references = ci.item_contains_object_references(enum_) %}
{%- include "EnumTemplate.kt" %}

{%- when Type::Error with (name) %}
{%- let error = ci.get_error_definition(name).unwrap() %}
{%- let contains_object_references = ci.item_contains_object_references(error) %}
{%- include "ErrorTemplate.kt" %}

{%- when Type::Record with (name) %}
{%- let rec = ci.get_record_definition(name).unwrap() %}
{%- let contains_object_references = ci.item_contains_object_references(rec) %}
{%- include "RecordTemplate.kt" %}

{%- when Type::Object with (name) %}
{%- let obj = ci.get_object_definition(name).unwrap() %}
{%- include "ObjectTemplate.kt" %}

{%- when Type::CallbackInterface with (name) %}
{%- let cbi = ci.get_callback_interface_definition(name).unwrap() %}
{%- include "CallbackInterfaceTemplate.kt" %}

{%- when Type::Optional with (inner_type) %}
{%- include "OptionalTemplate.kt" %}

{%- when Type::Sequence with (inner_type) %}
{%- include "SequenceTemplate.kt" %}

{%- when Type::Map with (inner_type) %}
{%- include "MapTemplate.kt" %}

{%- when Type::Custom with { name, builtin } %}
{%- let config = self.get_custom_type_config(name) %}
{%- include "CustomTypeTemplate.kt" %}

{%- else %}
{%- endmatch %}
