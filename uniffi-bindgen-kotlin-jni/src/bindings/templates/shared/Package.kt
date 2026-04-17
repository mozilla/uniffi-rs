@file:OptIn(kotlin.ExperimentalUnsignedTypes::class)
package uniffi

// In general, we prefer fully-qualified names to imports
// However, we do need to import extension methods
// since there's no other way to call them.
import kotlinx.coroutines.launch

class InternalException(message: kotlin.String) : Exception(message)

{% include "Async.kt" %}
{% include "FfiBuffer.kt" %}
{% include "LiftLower.kt" %}
{% include "Scaffolding.kt" %}
{% include "Interfaces.kt" %}
{% include "CallbackInterfaces.kt" %}

{%- for type_def in root.ffi_type_definitions() %}
{%- match type_def %}
{%- when TypeDefinition::Record(rec) %}
{% include "RecordFfi.kt" %}
{%- when TypeDefinition::Enum(en) %}
{% include "EnumFfi.kt" %}
{%- when TypeDefinition::Class(cls) %}
{% include "ClassFfi.kt" %}
{%- if let Some(cbi) = cls.callback_interface %}
{% include "CallbackInterfaceFfi.kt" %}
{%- endif %}
{%- when TypeDefinition::CallbackInterface(cbi) %}
{% include "CallbackInterfaceFfi.kt" %}
{%- when TypeDefinition::Custom(custom) %}
{% include "CustomFfi.kt" %}
{%- when TypeDefinition::Box(box_) %}
{% include "BoxFfi.kt" %}
{%- when TypeDefinition::Optional(opt) %}
{% include "OptionalFfi.kt" %}
{%- when TypeDefinition::Sequence(seq) %}
{% include "SequenceFfi.kt" %}
{%- when TypeDefinition::Map(map) %}
{% include "MapFfi.kt" %}
{%- when TypeDefinition::Set(set) %}
{% include "SetFfi.kt" %}
{%- when TypeDefinition::Timestamp(type_node) %}
{% include "TimestampFfi.kt" %}
{%- when TypeDefinition::Duration(type_node) %}
{% include "DurationFfi.kt" %}
{%- when TypeDefinition::Interface(_) %}
{%- endmatch %}
{%- endfor %}
