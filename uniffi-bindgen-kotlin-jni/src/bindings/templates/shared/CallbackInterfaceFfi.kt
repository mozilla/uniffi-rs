{%- let type_name = cbi.self_type.type_kt %}

private val {{ cbi.handle_map_kt() }} = HandleMap<{{ type_name }}>();

{%- for meth in cbi.methods %}
{%- if !meth.callable.is_async %}
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
{%- else %}
fun {{ meth.dispatch_fn_kt }}(
    uniffiHandle: kotlin.Long,
    uniffiOneshotHandle: kotlin.Long,
    {%- for ffi_arg in meth.callable.ffi_arguments() %}
    {{ ffi_arg.name_kt() }}: {{ ffi_arg.ty.type_kt() }},
    {%- endfor %}
) {
    // Using `GlobalScope` is labeled as a "delicate API" and generally discouraged in Kotlin programs, since it breaks structured concurrency.
    // However, our parent task is a Rust future, so we're going to need to break structure concurrency in any case.
    //
    // Uniffi does its best to support structured concurrency across the FFI.
    // If the Rust future is dropped, `uniffiForeignFutureDroppedCallbackImpl` is called, which will cancel the Kotlin coroutine if it's still running.
    @OptIn(kotlinx.coroutines.DelicateCoroutinesApi::class)
    kotlinx.coroutines.GlobalScope.launch uniffiCoroutineBlock@ {
        try {
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
                Scaffolding.{{ meth.callable.result.async_complete_error_fn() }}(
                    uniffiOneshotHandle,
                    {%- for (var, _) in throws_type.ffi_values_kt("uniffiErrLowered") %}
                    {{ var }},
                    {%- endfor %}
                )
                return@uniffiCoroutineBlock {{ meth.default_return_kt() }}
            }
            {%- endmatch %}

            {%- if let Some(return_type) = meth.callable.return_type() %}
            val uniffiReturnLowered = {{ return_type.lower_fn_kt() }}(uniffiReturn)
            Scaffolding.{{ meth.callable.result.async_complete_success_fn() }}(
                uniffiOneshotHandle,
                {%- for (var, _) in return_type.ffi_values_kt("uniffiReturnLowered") %}
                {{ var }},
                {%- endfor %}
            )
            {%- else %}
            Scaffolding.{{ meth.callable.result.async_complete_success_fn() }}(uniffiOneshotHandle)
            {%- endif %}
        } catch(e: Throwable) {
            Scaffolding.{{ meth.callable.result.async_complete_unexpected_error_fn() }}(uniffiOneshotHandle)
        }
    }
}
{%- endif %}
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
