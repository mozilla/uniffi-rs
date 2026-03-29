public fun {{ func.callable.name_kt() }}(
    {%- for arg in func.callable.arguments %}
    {{ arg.name_kt() }}: {{ arg.ty.type_kt }},
    {%- endfor %}
): {{ func.callable.return_type_kt() }} {
    val uniffiBuffer = uniffi.Scaffolding.ffiBufferNew()

    {%- for arg in func.callable.arguments %}
    {%- if loop.first %}
    val uniffiWriter = uniffi.FfiBufferCursor(uniffiBuffer)
    {%- endif %}
    uniffi.{{ arg.ty.write_fn_kt }}(uniffiWriter, {{ arg.name_kt() }})
    {%- endfor %}
    try {
        uniffi.Scaffolding.{{ func.jni_method_name }}(uniffiBuffer)
        {%- if let Some(return_ty) = func.callable.return_type %}
        val uniffiReader = uniffi.FfiBufferCursor(uniffiBuffer)
        return uniffi.{{ return_ty.read_fn_kt }}(uniffiReader)
        {%- endif %}
    } finally {
        uniffi.Scaffolding.ffiBufferFree(uniffiBuffer)
    }
}
