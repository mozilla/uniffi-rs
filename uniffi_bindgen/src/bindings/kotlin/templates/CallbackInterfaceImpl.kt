{%- let trait_impl=format!("uniffiCallbackInterface{}", name) %}

// Put the implementation in an object so we don't pollute the top-level namespace
internal object {{ trait_impl }} {
    {%- for (ffi_callback, meth) in vtable_methods.iter() %}
    {%- if !meth.is_async() %}
    internal object {{ meth.name()|var_name }}: UniffiCallbackMethod {
        override fun callback(uniffiFfiBuffer: Pointer) {
            var uniffiArgCursor = UniffiBufferCursor(uniffiFfiBuffer)
            val uniffiMethodHandle = UniffiFfiSerializerHandle.read(uniffiArgCursor)
            {%- for arg in meth.arguments() %}
            val {{ arg.name()|var_name }} = {{ arg|ffi_serializer_name(ci) }}.read(uniffiArgCursor);
            {%- endfor %}
            val uniffiObj = {{ ffi_converter_name }}.handleMap.get(uniffiMethodHandle)

            try {
                {%- match meth.throws_type() %}
                {%- when None %}
                {% if meth.return_type().is_some() %}val uniffiReturnValue = {% endif %}uniffiObj.{{ meth.name()|fn_name() }}(
                    {%- for arg in meth.arguments() %}
                    {{ arg|lift_fn }}({{ arg.name()|var_name }}),
                    {%- endfor %}
                )
                {%- when Some(error_type) %}
                {% if meth.return_type().is_some() %}val uniffiReturnValue = {% endif %}try {
                    uniffiObj.{{ meth.name()|fn_name() }}(
                        {%- for arg in meth.arguments() %}
                        {{ arg|lift_fn }}({{ arg.name()|var_name }}),
                        {%- endfor %}
                    )
                } catch(e: {{ error_type|type_name(ci) }}) {
                    val uniffiReturnCursor = UniffiBufferCursor(uniffiFfiBuffer)
                    UniffiFfiSerializerUniffiRustCallStatus.write(
                        uniffiReturnCursor,
                        UniffiRustCallStatus.create(UNIFFI_CALL_ERROR, {{ error_type|lower_fn }}(e))
                    )
                    return
                }
                {%- endmatch %}

                val uniffiReturnCursor = UniffiBufferCursor(uniffiFfiBuffer)
                // Default RustCallStatus signals success
                UniffiFfiSerializerUniffiRustCallStatus.write(uniffiReturnCursor, UniffiRustCallStatus.ByValue())
                {%- if let Some(return_type) = meth.return_type() %}
                {{ return_type|ffi_serializer_name(ci) }}.write(uniffiReturnCursor, {{ return_type|lower_fn }}(uniffiReturnValue))
                {%- endif %}
            } catch(e: kotlin.Exception) {
                val uniffiReturnCursor = UniffiBufferCursor(uniffiFfiBuffer)
                try { 
                    val err = {{ Type::String.borrow()|lower_fn }}(e.stackTraceToString())
                    UniffiFfiSerializerUniffiRustCallStatus.write(
                        uniffiReturnCursor,
                        UniffiRustCallStatus.create(UNIFFI_CALL_UNEXPECTED_ERROR, err)
                    )
                } catch(_: Throwable) {
                    // Exception serializing the error message, just use an empty RustBuffer.
                    UniffiFfiSerializerUniffiRustCallStatus.write(
                        uniffiReturnCursor,
                        UniffiRustCallStatus.create(UNIFFI_CALL_UNEXPECTED_ERROR, RustBuffer.ByValue())
                    )
                }
            }
        }
    }
    {%- else %}
    internal object {{ meth.name()|var_name }}: UniffiAsyncCallbackMethod {
        override fun callback(uniffiFfiBuffer: Pointer, uniffiCallback: UniffiForeignFutureCallback): UniffiForeignFutureDroppedCallback {
            var uniffiArgCursor = UniffiBufferCursor(uniffiFfiBuffer)
            val uniffiMethodHandle = UniffiFfiSerializerHandle.read(uniffiArgCursor)
            {%- for arg in meth.arguments() %}
            val {{ arg.name()|var_name }} = {{ arg|ffi_serializer_name(ci) }}.read(uniffiArgCursor);
            {%- endfor %}
            val uniffiFutureHandle = UniffiFfiSerializerHandle.read(uniffiArgCursor)
            val uniffiObj = {{ ffi_converter_name }}.handleMap.get(uniffiMethodHandle)

            return uniffiCallAsync(uniffiFfiBuffer) {
                val returnFfiBuffer = Memory(
                    {% if let Some(return_ty) = meth.return_type() %}{{ return_ty|ffi_serializer_name(ci) }}.size() + {% endif -%}
                    UniffiFfiSerializerLong.size() + UniffiFfiSerializerUniffiRustCallStatus.size()
                )
                try {
                    {%- match meth.throws_type() %}
                    {%- when None %}
                    {%- if meth.return_type().is_some() %}val uniffiReturnValue = {% endif %}uniffiObj.{{ meth.name()|fn_name() }}(
                        {%- for arg in meth.arguments() %}
                        {{ arg|lift_fn }}({{ arg.name()|var_name }}),
                        {%- endfor %}
                    )
                    {%- when Some(error_type) %}
                    {% if meth.return_type().is_some() %}val uniffiReturnValue = {% endif %}try {
                        uniffiObj.{{ meth.name()|fn_name() }}(
                            {%- for arg in meth.arguments() %}
                            {{ arg|lift_fn }}({{ arg.name()|var_name }}),
                            {%- endfor %}
                        )
                    } catch(e: {{ error_type|type_name(ci) }}) {
                        val uniffiReturnCursor = UniffiBufferCursor(returnFfiBuffer)
                        UniffiFfiSerializerHandle.write(uniffiReturnCursor, uniffiFutureHandle)
                        UniffiFfiSerializerUniffiRustCallStatus.write(
                            uniffiReturnCursor,
                            UniffiRustCallStatus.create(UNIFFI_CALL_ERROR, {{ error_type|lower_fn }}(e))
                        )
                        uniffiCallback.callback(returnFfiBuffer)
                        return@uniffiCallAsync
                    }
                    {%- endmatch %}

                    val uniffiReturnCursor = UniffiBufferCursor(returnFfiBuffer)
                    UniffiFfiSerializerHandle.write(uniffiReturnCursor, uniffiFutureHandle)
                    // Default RustCallStatus signals success
                    UniffiFfiSerializerUniffiRustCallStatus.write(uniffiReturnCursor, UniffiRustCallStatus.ByValue())
                    {%- if let Some(return_type) = meth.return_type() %}
                    {{ return_type|ffi_serializer_name(ci) }}.write(uniffiReturnCursor, {{ return_type|lower_fn }}(uniffiReturnValue))
                    {%- endif %}
                    uniffiCallback.callback(returnFfiBuffer)
                } catch(e: kotlin.Exception) {
                    val uniffiReturnCursor = UniffiBufferCursor(returnFfiBuffer)
                    try { 
                        val err = {{ Type::String.borrow()|lower_fn }}(e.stackTraceToString())
                        UniffiFfiSerializerHandle.write(uniffiReturnCursor, uniffiFutureHandle)
                        UniffiFfiSerializerUniffiRustCallStatus.write(
                            uniffiReturnCursor,
                            UniffiRustCallStatus.create(UNIFFI_CALL_UNEXPECTED_ERROR, err)
                        )
                    } catch(_: Throwable) {
                        // Exception serializing the error message, just use an empty RustBuffer.
                        UniffiFfiSerializerHandle.write(uniffiReturnCursor, uniffiFutureHandle)
                        UniffiFfiSerializerUniffiRustCallStatus.write(
                            uniffiReturnCursor,
                            UniffiRustCallStatus.create(UNIFFI_CALL_UNEXPECTED_ERROR, RustBuffer.ByValue())
                        )
                    }
                    uniffiCallback.callback(returnFfiBuffer)
                }
            }
        }
    }
    {%- endif %}
    {%- endfor %}

    internal object uniffiFree: UniffiCallbackMethod {
        override fun callback(uniffiFfiBuffer: Pointer) {
            val argCursor = UniffiBufferCursor(uniffiFfiBuffer)
            val handle = UniffiFfiSerializerHandle.read(argCursor)
            {{ ffi_converter_name }}.handleMap.remove(handle)
        }
    }

    internal object uniffiClone: UniffiCallbackMethod {
        override fun callback(uniffiFfiBuffer: Pointer) {
            val argCursor = UniffiBufferCursor(uniffiFfiBuffer)
            val handle = UniffiFfiSerializerHandle.read(argCursor)
            val clonedHandle = {{ ffi_converter_name }}.handleMap.clone(handle)
            val returnCursor = UniffiBufferCursor(uniffiFfiBuffer)
            UniffiFfiSerializerHandle.write(returnCursor, clonedHandle)
        }
    }

    internal var vtable = {{ vtable|ffi_type_name(ci) }}(
        uniffiFree,
        uniffiClone,
        {%- for (ffi_callback, meth) in vtable_methods.iter() %}
        {{ meth.name()|var_name() }},
        {%- endfor %}
    )

    // Registers the foreign callback with the Rust side.
    // This method is generated for each callback interface.
    internal fun register(lib: UniffiLib) {
        vtable.write()
        lib.{{ ffi_init_callback.pointer_ffi_name() }}(vtable.pointer)
    }
}
