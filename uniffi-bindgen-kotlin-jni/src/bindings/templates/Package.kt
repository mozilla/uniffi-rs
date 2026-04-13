package {{ package.name }}

{%- for import in package.imports %}
import {{ import }}
{%- endfor %}

{%- for type_def in package.type_definitions %}
{%- match type_def %}
{%- when TypeDefinition::Record(rec) %}
{% include "Record.kt" %}
{%- when TypeDefinition::Enum(en) %}
{% include "Enum.kt" %}
{%- when TypeDefinition::Interface(int) %}
{% include "Interface.kt" %}
{%- when TypeDefinition::Class(cls) %}
{% include "Class.kt" %}
{%- when TypeDefinition::Custom(custom) %}
{% include "Custom.kt" %}
{%- when TypeDefinition::CallbackInterface(_) %}
{%- when TypeDefinition::Box(_) %}
{%- when TypeDefinition::Optional(_) %}
{%- when TypeDefinition::Sequence(_) %}
{%- when TypeDefinition::Map(_) %}
{%- endmatch %}
{%- endfor %}

{%- for func in package.functions %}
{% include "Function.kt" %}
{%- endfor %}
