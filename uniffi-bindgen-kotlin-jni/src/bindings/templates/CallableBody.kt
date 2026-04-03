val uniffiBuffer = uniffi.Scaffolding.ffiBufferNew()
val uniffiWriter = uniffi.FfiBufferCursor(uniffiBuffer)

{%- if let Some(receiver_type) = callable.receiver_type() %}
uniffi.{{ receiver_type.write_fn_kt }}(uniffiWriter, this)
{%- endif %}

{%- for arg in callable.arguments %}
uniffi.{{ arg.ty.write_fn_kt }}(uniffiWriter, {{ arg.name_kt() }})
{%- endfor %}
try {
    uniffi.Scaffolding.{{ jni_method_name }}(uniffiBuffer)
    {%- if callable.is_primary_constructor() %}
    val uniffiReader = uniffi.FfiBufferCursor(uniffiBuffer)
    this.uniffiHandle = uniffi.readLong(uniffiReader)
    {%- else if let Some(return_ty) = callable.return_type %}
    val uniffiReader = uniffi.FfiBufferCursor(uniffiBuffer)
    return uniffi.{{ return_ty.read_fn_kt }}(uniffiReader)
    {%- endif %}
} finally {
    uniffi.Scaffolding.ffiBufferFree(uniffiBuffer)
}
