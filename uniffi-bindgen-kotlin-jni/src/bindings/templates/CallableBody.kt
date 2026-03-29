{%- for arg in callable.arguments %}
val uniffiArgLowered{{ loop.index0 }} = uniffi.{{ arg.ty.lower_fn_kt() }}({{ arg.name_kt() }})
{%- endfor %}

val uniffiReturn = uniffi.Scaffolding.{{ jni_method_name }}(
    {%- for arg in callable.arguments %}
    uniffiArgLowered{{ loop.index0 }},
    {%- endfor %}
)
{%- match callable.return_ffi %}
{%- when ReturnFfi::Primitive { type_node, .. } %}
return uniffi.{{ type_node.lift_fn_kt() }}(uniffiReturn)
{% when ReturnFfi::Void %}
{%- endmatch %}
