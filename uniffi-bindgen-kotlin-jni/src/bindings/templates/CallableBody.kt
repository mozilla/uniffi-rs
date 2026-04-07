val uniffiBuffer = uniffi.Scaffolding.ffiBufferNew()
val uniffiWriter = uniffi.FfiBufferCursor(uniffiBuffer)
try {
    {%- if let Some(receiver_type) = callable.receiver_type() %}
    uniffi.{{ receiver_type.write_fn_kt }}(uniffiWriter, this)
    {%- endif %}

    {%- for arg in callable.arguments %}
    uniffi.{{ arg.ty.write_fn_kt }}(uniffiWriter, {{ arg.name_kt() }})
    {%- endfor %}

    {% if !callable.is_async %}

    uniffi.Scaffolding.{{ jni_method_name }}(uniffiBuffer)
    {%- if callable.is_primary_constructor() %}
    val uniffiReader = uniffi.FfiBufferCursor(uniffiBuffer)
    this.uniffiHandle = uniffi.readLong(uniffiReader)
    {%- else if let Some(return_ty) = callable.return_type %}
    val uniffiReader = uniffi.FfiBufferCursor(uniffiBuffer)
    return uniffi.{{ return_ty.read_fn_kt }}(uniffiReader)
    {%- endif %}

    {%- else  %}
    val uniffiFuture = uniffi.Scaffolding.{{ jni_method_name }}(uniffiBuffer)
    val uniffiCode = uniffi.awaitFuture(uniffiFuture);
    when(uniffiCode) {
        uniffi.UNIFFI_RUST_FUTURE_CANCELLED -> throw kotlin.coroutines.cancellation.CancellationException()
        uniffi.UNIFFI_RUST_FUTURE_COMPLETE -> {
            {% if let Some(return_ty) = callable.return_type %}
            val uniffiReader = uniffi.FfiBufferCursor(uniffiBuffer)
            return uniffi.{{ return_ty.read_fn_kt }}(uniffiReader)
            {%- endif %}
        }
        {% if let Some(throws_ty) = callable.throws_type %}
        uniffi.UNIFFI_RUST_FUTURE_ERROR -> {
            val uniffiReader = uniffi.FfiBufferCursor(uniffiBuffer)
            throw uniffi.{{ throws_ty.read_fn_kt }}(uniffiReader)
        }
        {%- endif %}
        else -> throw uniffi.InternalException("Error polling Rust future (code: $uniffiCode)")
    }
    {%- endif %}
} finally {
    uniffi.Scaffolding.ffiBufferFree(uniffiBuffer)
}

