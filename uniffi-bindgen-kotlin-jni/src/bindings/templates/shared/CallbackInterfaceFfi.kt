{%- let type_name = cbi.self_type.type_kt %}

private val {{ cbi.handle_map_kt() }} = HandleMap<{{ type_name }}>();

{%- for meth in cbi.methods %}
{%- if !meth.callable.is_async %}
fun {{ meth.dispatch_fn_kt }}(uniffiHandle: Long, uniffiBuffer: Long) {
    val uniffiObj = {{ cbi.handle_map_kt() }}.get(uniffiHandle)
    {% for a in meth.callable.arguments %}
    {%- if loop.first %}
    val uniffiReader = FfiBufferCursor(uniffiBuffer)
    {%- endif %}
    val {{ a.name_kt() }} = {{ a.ty.read_fn_kt }}(uniffiReader)
    {%- endfor %}
    {%- match meth.callable.throws_type %}
    {%- when None %}
    val uniffiReturn = uniffiObj.{{ meth.callable.name_kt() }}(
        {%- for a in meth.callable.arguments %}
        {{ a.name_kt() }},
        {%- endfor %}
    )
    {%- when Some(throws_ty) %}
    val uniffiReturn = try {
        uniffiObj.{{ meth.callable.name_kt() }}(
            {%- for a in meth.callable.arguments %}
            {{ a.name_kt() }},
            {%- endfor %}
        )
    } catch(e: {{ throws_ty.type_kt }}) {
        val uniffiWriter = FfiBufferCursor(uniffiBuffer)
        {{ throws_ty.write_fn_kt }}(uniffiWriter, e)
        throw uniffi.CallbackException(uniffiBuffer)
    }
    {%- endmatch %}
    {% if let Some(return_ty) = meth.callable.return_type %}
    val uniffiWriter = FfiBufferCursor(uniffiBuffer)
    {{ return_ty.write_fn_kt }}(uniffiWriter, uniffiReturn)
    {%- endif %}
}
{%- else %}
fun {{ meth.dispatch_fn_kt }}(uniffiHandle: Long, uniffiBuffer: Long) {
    throw RuntimeException("TODO")
}
{%- endif %}
{%- endfor %}

fun {{ cbi.free_fn_kt() }}(handle: Long) {
    {{ cbi.handle_map_kt() }}.remove(handle)
}

// Note: no read function, since callback interfaces can't be passed back from Rust to Kotlin

fun {{ cbi.self_type.write_fn_kt }}(cursor: FfiBufferCursor, value: {{ type_name }}) {
    writeLong(cursor, {{ cbi.handle_map_kt() }}.insert(value))
}

