{%- let type_name = cbi.self_type.type_kt %}

private val {{ cbi.handle_map_kt() }} = HandleMap<{{ type_name }}>();

{%- for meth in cbi.methods %}
fun {{ meth.dispatch_fn_kt }}(
    uniffiHandle: kotlin.Long,
    {%- if meth.has_return_pointer() %}
    uniffiReturnPointer: kotlin.Long,
    {%- endif %}
    {%- for ffi_arg in meth.callable.ffi_arguments() %}
    {{ ffi_arg.name_kt() }}: {{ ffi_arg.ty.type_kt() }},
    {%- endfor %}
)
{%- match meth.callable.return_ffi() %}
{%- when ReturnFfi::Primitive { ffi_type, .. } %} : {{ ffi_type.type_kt() }}
{%- else %}
{%- endmatch %}
{
    val uniffiObj = {{ cbi.handle_map_kt() }}.get(uniffiHandle)

    {%- for arg in meth.callable.arguments %}
    val {{ arg.name_kt() }} = uniffi.{{ arg.ty.lift_fn_kt() }}(
        {%- for ffi_arg in arg.ffi_args() %}
        {{ ffi_arg.name_kt() }},
        {%- endfor %}
    )
    {%- endfor %}

    {%- match meth.callable.throws_type() %}
    {%- when None %}
    val uniffiReturn = uniffiObj.{{ meth.callable.name_kt() }}(
        {%- for a in meth.callable.arguments %}
        {{ a.name_kt() }},
        {%- endfor %}
    )
    {%- when Some(throws_type) %}
    val uniffiReturn = try {
        uniffiObj.{{ meth.callable.name_kt() }}(
            {%- for a in meth.callable.arguments %}
            {{ a.name_kt() }},
            {%- endfor %}
        )
    } catch(uniffiErr: {{ throws_type.type_kt }}) {
        val uniffiErrLowered = {{ throws_type.lower_fn_kt() }}(uniffiErr)
        Scaffolding.{{ meth.callable.result.set_callback_err_fn() }}(
            uniffiReturnPointer,
            {%- for (var, _) in throws_type.ffi_values_kt("uniffiErrLowered") %}
            {{ var }},
            {%- endfor %}
        )
        return {{ meth.default_return_kt() }}
    }
    {%- endmatch %}

    {%- match meth.callable.return_ffi() %}
    {%- when ReturnFfi::Primitive { type_node, ffi_type } %}
    return {{ type_node.lower_fn_kt() }}(uniffiReturn)
    {%- when ReturnFfi::Deconstruct { type_node, ffi_types } %}
    val uniffiReturnLowered = {{ type_node.lower_fn_kt() }}(uniffiReturn)
    Scaffolding.{{ meth.callable.result.set_callback_return_fn() }}(
        uniffiReturnPointer,
        {%- for _ in ffi_types %}
        uniffiReturnLowered.v{{ loop.index0 }},
        {%- endfor %}
    )
    {%- when ReturnFfi::Void %}
    {%- endmatch %}
}
{%- endfor %}

fun {{ cbi.free_fn_kt() }}(handle: kotlin.Long) {
    {{ cbi.handle_map_kt() }}.remove(handle)
}

fun {{ cbi.self_type.lower_fn_kt() }}(value: {{ type_name }}): kotlin.Long {
    return {{ cbi.handle_map_kt() }}.insert(value)
}

fun {{ cbi.self_type.write_fn_kt() }}(buf: java.nio.ByteBuffer, offset: kotlin.Int, value: {{ type_name }}) {
    writeLong(buf, offset, {{ cbi.handle_map_kt() }}.insert(value))
}

// Note: no read/lift function, since callback interfaces can't be passed back from Rust to Kotlin
