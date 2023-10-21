{% if self.include_once_check("CallbackInterfaceRuntime.kt") %}{% include "CallbackInterfaceRuntime.kt" %}{% endif %}

// Implement the foreign callback handler for {{ interface_name }}
internal class {{ callback_handler_class }} : ForeignCallback {
    @Suppress("TooGenericExceptionCaught")
    override fun invoke(handle: Handle, method: Int, argsData: Pointer, argsLen: Int, outBuf: RustBufferByReference): Int {
        val cb = {{ ffi_converter_name }}.handleMap.get(handle)
        return when (method) {
            IDX_CALLBACK_FREE -> {
                {{ ffi_converter_name }}.handleMap.remove(handle)

                // Successful return
                // See docs of ForeignCallback in `uniffi_core/src/ffi/foreigncallbacks.rs`
                UNIFFI_CALLBACK_SUCCESS
            }
            {% for meth in methods.iter() -%}
            {% let method_name = format!("invoke_{}", meth.name())|fn_name -%}
            {{ loop.index }} -> {
                // Call the method, write to outBuf and return a status code
                // See docs of ForeignCallback in `uniffi_core/src/ffi/foreigncallbacks.rs` for info
                try {
                    this.{{ method_name }}(cb, argsData, argsLen, outBuf)
                } catch (e: Throwable) {
                    // Unexpected error
                    try {
                        // Try to serialize the error into a string
                        outBuf.setValue({{ Type::String.borrow()|ffi_converter_name }}.lower(e.toString()))
                    } catch (e: Throwable) {
                        // If that fails, then it's time to give up and just return
                    }
                    UNIFFI_CALLBACK_UNEXPECTED_ERROR
                }
            }
            {% endfor %}
            else -> {
                // An unexpected error happened.
                // See docs of ForeignCallback in `uniffi_core/src/ffi/foreigncallbacks.rs`
                try {
                    // Try to serialize the error into a string
                    outBuf.setValue({{ Type::String.borrow()|ffi_converter_name }}.lower("Invalid Callback index"))
                } catch (e: Throwable) {
                    // If that fails, then it's time to give up and just return
                }
                UNIFFI_CALLBACK_UNEXPECTED_ERROR
            }
        }
    }

    {% for meth in methods.iter() -%}
    {% let method_name = format!("invoke_{}", meth.name())|fn_name %}
    @Suppress("UNUSED_PARAMETER")
    private fun {{ method_name }}(kotlinCallbackInterface: {{ interface_name }}, argsData: Pointer, argsLen: Int, outBuf: RustBufferByReference): Int {
        {%- if meth.arguments().len() > 0 %}
        val argsBuf = argsData.getByteBuffer(0, argsLen.toLong()).also {
            it.order(ByteOrder.BIG_ENDIAN)
        }
        {%- endif %}

        {%- match meth.return_type() %}
        {%- when Some with (return_type) %}
        fun makeCall() : Int {
            val returnValue = kotlinCallbackInterface.{{ meth.name()|fn_name }}(
                {%- for arg in meth.arguments() %}
                {{ arg|read_fn }}(argsBuf)
                {% if !loop.last %}, {% endif %}
                {%- endfor %}
            )
            {{ return_type|unchecked_cast_annotation_if_needed -}}
            outBuf.setValue({{ return_type|ffi_converter_name }}.lowerIntoRustBuffer(returnValue{{ return_type|downcast_if_needed }}))
            return UNIFFI_CALLBACK_SUCCESS
        }
        {%- when None %}
        fun makeCall() : Int {
            kotlinCallbackInterface.{{ meth.name()|fn_name }}(
                {%- for arg in meth.arguments() %}
                {{ arg|read_fn }}(argsBuf)
                {%- if !loop.last %}, {% endif %}
                {%- endfor %}
            )
            return UNIFFI_CALLBACK_SUCCESS
        }
        {%- endmatch %}

        {%- match meth.throws_type() %}
        {%- when None %}
        fun makeCallAndHandleError() : Int = makeCall()
        {%- when Some(error_type) %}
        fun makeCallAndHandleError()  : Int = try {
            makeCall()
        } catch (e: {{ error_type|error_type_name }}) {
            // Expected error, serialize it into outBuf
            outBuf.setValue({{ error_type|ffi_converter_name }}.lowerIntoRustBuffer(e))
            UNIFFI_CALLBACK_ERROR
        }
        {%- endmatch %}

        return makeCallAndHandleError()
    }
    {% endfor %}

    // Registers the foreign callback with the Rust side.
    // This method is generated for each callback interface.
    internal fun register(lib: _UniFFILib) {
        lib.{{ ffi_init_callback.name() }}(this)
    }
}

internal val {{ callback_handler_obj }} = {{ callback_handler_class }}()
