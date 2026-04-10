// Skip clippy checks for the generate bindings
#[allow(clippy::unused_unit)]
#[allow(clippy::let_unit_value)]
#[allow(unused)]
mod uniffi_bindgen_kotlin_jni_scaffolding {
    use ::uniffi_bindgen_kotlin_jni_runtime as uniffi_jni;
    use ::uniffi_bindgen_kotlin_jni_runtime::uniffi;

    {% filter indent(4) %}{% include "shared.rs" %}{% endfilter %}

    {%- for type_def in root.ffi_type_definitions() %}
    {%- match type_def %}
    {%- when TypeDefinition::Record(rec) %}
    {%- filter indent(4) %}{% include "record.rs" %}{% endfilter %}
    {%- if rec.self_type.is_used_as_error %}
    {%- let type_node = rec.self_type %}
    {%- filter indent(4) %}{% include "throw_error_fn.rs" %}{% endfilter %}
    {%- endif %}
    {%- when TypeDefinition::Enum(en) %}
    {%- filter indent(4) %}{% include "enum.rs" %}{% endfilter %}
    {%- if en.self_type.is_used_as_error %}
    {%- let type_node = en.self_type %}
    {%- filter indent(4) %}{% include "throw_error_fn.rs" %}{% endfilter %}
    {%- endif %}
    {%- when TypeDefinition::Class(cls) %}
    {%- filter indent(4) %}{% include "class.rs" %}{% endfilter %}
    {%- if cls.self_type.is_used_as_error %}
    {%- let type_node = cls.self_type %}
    {%- filter indent(4) %}{% include "throw_error_fn.rs" %}{% endfilter %}
    {%- endif %}
    {%- if let Some(cbi) = cls.callback_interface %}
    {%- filter indent(4) %}{% include "callback_interface.rs" %}{% endfilter %}
    {%- endif %}
    {%- when TypeDefinition::CallbackInterface(cbi) %}
    {%- filter indent(4) %}{% include "callback_interface.rs" %}{% endfilter %}
    {%- when TypeDefinition::Custom(custom) %}
    {%- filter indent(4) %}{% include "custom.rs" %}{% endfilter %}
    {%- when TypeDefinition::Optional(opt) %}
    {%- filter indent(4) %}{% include "optional.rs" %}{% endfilter %}
    {%- when TypeDefinition::Sequence(seq) %}
    {%- filter indent(4) %}{% include "sequence.rs" %}{% endfilter %}
    {%- when TypeDefinition::Map(map) %}
    {%- filter indent(4) %}{% include "map.rs" %}{% endfilter %}
    {%- when TypeDefinition::Interface(_) %}
    {%- endmatch %}
    {%- endfor %}

    {%- for package in root.packages %}
    {%- for scaffolding_function in package.scaffolding_functions %}
    {%- filter indent(4) %}{% include "scaffolding_function.rs" %}{% endfilter %}
    {%- endfor %}
    {%- endfor %}
}
