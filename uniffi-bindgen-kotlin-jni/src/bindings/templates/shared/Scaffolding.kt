object Scaffolding {
    @JvmStatic external fun ffiBufferCheckSupport()
    @JvmStatic external fun ffiBufferAlloc(capacity: kotlin.Int): java.nio.ByteBuffer
    @JvmStatic external fun ffiBufferFree(buf: java.nio.ByteBuffer)
    @JvmStatic external fun ffiBufferReadString(buf: java.nio.ByteBuffer, offset: kotlin.Int): kotlin.String
    @JvmStatic external fun ffiBufferWriteString(buf: java.nio.ByteBuffer, offset: kotlin.Int, value: kotlin.String)
    @JvmStatic external fun ffiBufferReadBuffer(buf: java.nio.ByteBuffer, offset: kotlin.Int): java.nio.ByteBuffer
    @JvmStatic external fun ffiBufferWriteBuffer(buf: java.nio.ByteBuffer, offset: kotlin.Int, value: java.nio.ByteBuffer)

    {%- for (jni_method_name, callable) in root.jni_methods() %}
    @JvmName("{{ jni_method_name }}")
    @JvmStatic external fun {{ jni_method_name }}(
        {%- for ffi_arg in callable.ffi_arguments_including_receiver() %}
        {{ ffi_arg.name_kt() }}: {{ ffi_arg.ty.type_kt() }},
        {%- endfor %}
    )
    {%- match callable.return_ffi() %}
    {%- when ReturnFfi::Primitive { ffi_type, .. } %} : {{ ffi_type.type_kt() }}
    {%- when ReturnFfi::Deconstruct { type_node, .. } %} : {{ type_node.type_kt }}
    {%- when ReturnFfi::Void %}
    {%- endmatch %}
    {%- endfor %}

    {%- for cls in root.classes() %}
    @JvmStatic external fun {{ cls.jni_clone_name() }}(handle: kotlin.Long): kotlin.Long
    @JvmStatic external fun {{ cls.jni_free_name() }}(handle: kotlin.Long)
    {%- endfor  %}

    {%- for result in root.kotlin_sync_callable_results() %}
    {%- if let Some(return_type) = result.return_type %}
    {%- if !return_type.lowers_to_primitive() %}
    @JvmStatic external fun {{ result.set_callback_return_fn() }}(
        resultPtr: kotlin.Long,
        {%- for ffi_type in return_type.ffi_types %}
        v{{ loop.index0 }}: {{ ffi_type.type_kt() }},
        {%- endfor %}
    )
    {%- endif %}
    {%- endif %}

    {%- if let Some(throws_type) = result.throws_type %}
    @JvmStatic external fun {{ result.set_callback_err_fn() }}(
        resultPtr: kotlin.Long,
        {%- for ffi_type in throws_type.ffi_types %}
        v{{ loop.index0 }}: {{ ffi_type.type_kt() }},
        {%- endfor %}
    )
    {%- endif %}
    {%- endfor %}

    init {
        System.loadLibrary("{{ cdylib }}")
        Scaffolding.ffiBufferCheckSupport()
    }
}
