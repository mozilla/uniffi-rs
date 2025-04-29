{%- for type_def in type_definitions %}

{#
 # Map `Type` instances to an include statement for that type.
 #
 # There is a companion match in `PythonCodeOracle::create_code_type()` which performs a similar function for the
 # Rust code.
 #
 #   - When adding additional types here, make sure to also add a match arm to that function.
 #   - To keep things manageable, let's try to limit ourselves to these 2 mega-matches
 #}
{%- match type_def %}

{%- when TypeDefinition::Simple(type_node) %}
{%- match type_node.ty %}

{%- when Type::Boolean %}
{%- include "BooleanHelper.py" %}

{%- when Type::Int8 %}
{%- include "Int8Helper.py" %}

{%- when Type::Int16 %}
{%- include "Int16Helper.py" %}

{%- when Type::Int32 %}
{%- include "Int32Helper.py" %}

{%- when Type::Int64 %}
{%- include "Int64Helper.py" %}

{%- when Type::UInt8 %}
{%- include "UInt8Helper.py" %}

{%- when Type::UInt16 %}
{%- include "UInt16Helper.py" %}

{%- when Type::UInt32 %}
{%- include "UInt32Helper.py" %}

{%- when Type::UInt64 %}
{%- include "UInt64Helper.py" %}

{%- when Type::Float32 %}
{%- include "Float32Helper.py" %}

{%- when Type::Float64 %}
{%- include "Float64Helper.py" %}

{%- when Type::String %}
{%- include "StringHelper.py" %}

{%- when Type::Bytes %}
{%- include "BytesHelper.py" %}

{%- when Type::Timestamp %}
{%- include "TimestampHelper.py" %}

{%- when Type::Duration %}
{%- include "DurationHelper.py" %}

{%- else %}
{# Type::Simple shouldn't hold any other Type variants #}
{%- endmatch %}

{%- when TypeDefinition::Optional(opt) %}
{%- include "OptionalTemplate.py" %}

{%- when TypeDefinition::Sequence(seq) %}
{%- include "SequenceTemplate.py" %}

{%- when TypeDefinition::Map(map) %}
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

{%- when TypeDefinition::Interface(int) %}
{%- include "InterfaceTemplate.py" %}


{%- when TypeDefinition::CallbackInterface(cbi) %}
{%- include "CallbackInterfaceTemplate.py" %}

{%- when TypeDefinition::Custom(custom) %}
{%- include "CustomType.py" %}

{%- else %}
{%- endmatch %}
{%- endfor %}
