{%- import "macros.dart" as dart %}
{%- for type_ in ci.iter_types() %}
{%- let type_name = type_|type_name %}
{%- let ffi_converter_name = type_|ffi_converter_name %}
{%- let canonical_type_name = type_|canonical_name %}
{%- let contains_object_references = ci.item_contains_object_references(type_) %}

{#
 # Map `Type` instances to an include statement for that type.
 #
 # There is a companion match in `DartCodeOracle::create_code_type()` which performs a similar function for the
 # Rust code.
 #
 #   - When adding additional types here, make sure to also add a match arm to that function.
 #   - To keep things manageable, let's try to limit ourselves to these 2 mega-matches
 #}
{%- match type_ %}

{%- when Type::Boolean %}
{%- include "BooleanHelper.dart" %}

{%- when Type::String %}
{%- include "StringHelper.dart" %}

{%- when Type::UInt64 %}
{%- include "UInt64Helper.dart" %}

{# comment

{%- when Type::Int8 %}
{%- include "Int8Helper.dart" %}

{%- when Type::Int16 %}
{%- include "Int16Helper.dart" %}

{%- when Type::Int32 %}
{%- include "Int32Helper.dart" %}

{%- when Type::Int64 %}
{%- include "Int64Helper.dart" %}

{%- when Type::UInt8 %}
{%- include "UInt8Helper.dart" %}

{%- when Type::UInt16 %}
{%- include "UInt16Helper.dart" %}

{%- when Type::UInt32 %}
{%- include "UInt32Helper.dart" %}


{%- when Type::Float32 %}
{%- include "Float32Helper.dart" %}

{%- when Type::Float64 %}
{%- include "Float64Helper.dart" %}

{%- when Type::Timestamp %}
{%- include "TimestampHelper.dart" %}

{%- when Type::Duration %}
{%- include "DurationHelper.dart" %}

{%- when Type::CallbackInterface(name) %}
{%- include "CallbackInterfaceTemplate.dart" %}

{%- when Type::Custom { name, builtin } %}
{%- include "CustomType.dart" %}

{%- when Type::Error(name) %}
{%- include "ErrorTemplate.dart" %}

#}

{%- when Type::Enum(name) %}
{%- include "EnumTemplate.dart" %}

{%- when Type::Object(name) %}
{%- include "ObjectTemplate.dart" %}

{# comment

{%- when Type::Record(name) %}
{%- include "RecordTemplate.dart" %}

{%- when Type::Optional(inner_type) %}
{%- include "OptionalTemplate.dart" %}

{%- when Type::Sequence(inner_type) %}
{%- include "SequenceTemplate.dart" %}

{%- when Type::Map(key_type, value_type) %}
{%- include "MapTemplate.dart" %}


#}
{%- else %}
{%- endmatch %}
{%- endfor %}
