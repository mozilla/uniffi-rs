// Skip clippy checks for the generate bindings
#[allow(clippy::unused_unit)]
#[allow(clippy::let_unit_value)]
#[allow(clippy::type_complexity)]
#[allow(clippy::identity_op)]
#[allow(clippy::needless_question_mark)]
#[allow(clippy::unit_arg)]
#[allow(clippy::too_many_arguments)]
#[allow(unused)]
mod uniffi_bindgen_kotlin_jni_scaffolding {
    use ::uniffi_bindgen_kotlin_jni_runtime as uniffi_jni;
    use ::uniffi_bindgen_kotlin_jni_runtime::uniffi;
    use ::uniffi_bindgen_kotlin_jni_runtime::JniResultExt;

    {%- filter indent(4) %}{% include "async.rs" %}{% endfilter %}
    {%- filter indent(4) %}{% include "callback_interface_shared.rs" %}{% endfilter %}
    {%- filter indent(4) %}{% include "ffi_buffer.rs" %}{% endfilter %}
    {%- filter indent(4) %}{% include "jni_support.rs" %}{% endfilter %}
    {%- filter indent(4) %}{% include "lift_lower.rs" %}{% endfilter %}

    {%- for (jni_method_name, callable) in root.jni_methods() %}
    {%- filter indent(4) %}{% include "jni_method.rs" %}{% endfilter %}
    {%- endfor %}

    {%- for type_def in root.ffi_type_definitions() %}
    {%- match type_def %}
    {%- when TypeDefinition::Record(rec) %}
    {%- filter indent(4) %}{% include "record.rs" %}{% endfilter %}
    {%- when TypeDefinition::Enum(en) %}
    {%- filter indent(4) %}{% include "enum.rs" %}{% endfilter %}
    {%- when TypeDefinition::Class(cls) %}
    {%- filter indent(4) %}{% include "class.rs" %}{% endfilter %}
    {%- if let Some(cbi) = cls.callback_interface %}
    {%- filter indent(4) %}{% include "callback_interface.rs" %}{% endfilter %}
    {%- endif %}
    {%- when TypeDefinition::CallbackInterface(cbi) %}
    {%- filter indent(4) %}{% include "callback_interface.rs" %}{% endfilter %}
    {%- when TypeDefinition::Custom(custom) %}
    {%- filter indent(4) %}{% include "custom.rs" %}{% endfilter %}
    {%- when TypeDefinition::Box(box_) %}
    {%- filter indent(4) %}{% include "box.rs" %}{% endfilter %}
    {%- when TypeDefinition::Optional(opt) %}
    {%- filter indent(4) %}{% include "optional.rs" %}{% endfilter %}
    {%- when TypeDefinition::Sequence(seq) %}
    {%- filter indent(4) %}{% include "sequence.rs" %}{% endfilter %}
    {%- when TypeDefinition::Map(map) %}
    {%- filter indent(4) %}{% include "map.rs" %}{% endfilter %}
    {%- when TypeDefinition::Set(set) %}
    {%- filter indent(4) %}{% include "set.rs" %}{% endfilter %}
    {%- when TypeDefinition::Interface(_) %}
    {%- endmatch %}
    {%- endfor %}
}
