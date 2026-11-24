{%- for type_def in type_definitions %}
{%- match type_def %}
{%- when TypeDefinition::Record(rec) %}
{% include "Record.kt" %}
{%- when TypeDefinition::Enum(en) %}
{% include "Enum.kt" %}
{%- when TypeDefinition::Class(cls) %}
{% include "Class.kt" %}
{%- if let Some(cbi) = cls.callback_interface %}
{% include "CallbackInterface.kt" %}
{%- endif %}
{%- when TypeDefinition::CallbackInterface(cbi) %}
{% include "CallbackInterface.kt" %}
{%- when TypeDefinition::Custom(custom) %}
{% include "Custom.kt" %}
{%- when TypeDefinition::Box(box_) %}
{% include "Box.kt" %}
{%- when TypeDefinition::Optional(opt) %}
{% include "Optional.kt" %}
{%- when TypeDefinition::Sequence(seq) %}
{% include "Sequence.kt" %}
{%- when TypeDefinition::Map(map) %}
{% include "Map.kt" %}
{%- when TypeDefinition::Set(set) %}
{% include "Set.kt" %}
{%- when TypeDefinition::Simple(type_node) %}
{%- match type_node.ty %}
{%- when Type::Timestamp %}
{% include "Timestamp.kt" %}
{%- when Type::Duration %}
{% include "Duration.kt" %}
{%- else %}
{%- endmatch %}
{%- when TypeDefinition::External(ext) %}
{% include "External.kt" %}
{%- endmatch %}
{%- endfor %}
