{%- for arg in callable.arguments_including_receiver() %}
val uniffiArgLowered{{ arg.index }} = uniffi.{{ arg.lower_fn_kt() }}({{ arg.name_kt() }})
{%- endfor %}

val uniffiReturn = uniffi.Scaffolding.{{ jni_method_name }}(
    {%- for arg in callable.arguments_including_receiver() %}
    {%- for (var, _) in arg.ty.ffi_values_kt(format!("uniffiArgLowered{}", arg.index)) %}
    {{ var }},
    {%- endfor %}
    {%- endfor %}
)
{%- if callable.is_primary_constructor() %}
this.uniffiHandle = uniffiReturn
{%- else %}
{%- match callable.return_ffi %}
{%- when ReturnFfi::Primitive { type_node, .. } %}
return uniffi.{{ type_node.lift_fn_kt() }}(uniffiReturn)
{%- when ReturnFfi::Deconstruct { .. } %}
return uniffiReturn
{% when ReturnFfi::Void %}
{%- endmatch %}
{%- endif %}
