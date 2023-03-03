{%- let cbi = ci.get_callback_interface_definition(name).unwrap() %}
{%- let type_name = cbi|type_name %}
{%- let foreign_callback = format!("ForeignCallback{}", canonical_type_name) %}

{% if self.include_once_check("CallbackInterfaceRuntime.kt") %}{% include "CallbackInterfaceRuntime.kt" %}{% endif %}
{{- self.add_import("java.util.concurrent.atomic.AtomicLong") }}
{{- self.add_import("java.util.concurrent.locks.ReentrantLock") }}
{{- self.add_import("kotlin.concurrent.withLock") }}

// Declaration and FfiConverters for {{ type_name }} Callback Interface

public interface {{ type_name }} {
    {% for meth in cbi.methods() -%}
    fun {{ meth.name()|fn_name }}({% call kt::arg_list_decl(meth) %})
    {%- match meth.return_type() -%}
    {%- when Some with (return_type) %}: {{ return_type|type_name -}}
    {%- else -%}
    {%- endmatch %}
    {% endfor %}
}

// The ForeignCallback that is passed to Rust.
{%- if new_callback_interface_abi %}
internal class {{ foreign_callback }} : ForeignCallback {
    @Suppress("TooGenericExceptionCaught")
    override fun invoke(handle: Handle, method: Int, args: RustBuffer.ByReference, outBuf: RustBufferByReference): Int {
{%- else %}
internal class {{ foreign_callback }} : ForeignCallback {
    @Suppress("TooGenericExceptionCaught")
    override fun invoke(handle: Handle, method: Int, args: RustBuffer.ByValue, outBuf: RustBufferByReference): Int {
{%- endif %}
        val cb = {{ ffi_converter_name }}.lift(handle)
        return when (method) {
            IDX_CALLBACK_FREE -> {
                {{ ffi_converter_name }}.drop(handle)
                // No return value.
                // See docs of ForeignCallback in `uniffi/src/ffi/foreigncallbacks.rs`
                0
            }
            {% for meth in cbi.methods() -%}
            {% let method_name = format!("invoke_{}", meth.name())|fn_name -%}
            {{ loop.index }} -> {
                // Call the method, write to outBuf and return a status code
                // See docs of ForeignCallback in `uniffi/src/ffi/foreigncallbacks.rs` for info
                try {
                    this.{{ method_name }}(cb, args, outBuf)
                } catch (e: Throwable) {
                    // Unexpected error
                    try {
                        // Try to serialize the error into a string
                        outBuf.setValue({{ Type::String.borrow()|ffi_converter_name }}.lower(e.toString()))
                    } catch (e: Throwable) {
                        // If that fails, then it's time to give up and just return
                    }
                    -1
                }
            }
            {% endfor %}
            else -> {
                // An unexpected error happened.
                // See docs of ForeignCallback in `uniffi/src/ffi/foreigncallbacks.rs`
                try {
                    // Try to serialize the error into a string
                    outBuf.setValue({{ Type::String.borrow()|ffi_converter_name }}.lower("Invalid Callback index"))
                } catch (e: Throwable) {
                    // If that fails, then it's time to give up and just return
                }
                -1
            }
        }
    }

    {% for meth in cbi.methods() -%}
    {% let method_name = format!("invoke_{}", meth.name())|fn_name %}
    {%- if new_callback_interface_abi %}
    @Suppress("UNUSED_PARAMETER")
    private fun {{ method_name }}(kotlinCallbackInterface: {{ type_name }}, args: RustBuffer.ByReference, outBuf: RustBufferByReference): Int {
    {%- else %}
    @Suppress("UNUSED_PARAMETER")
    private fun {{ method_name }}(kotlinCallbackInterface: {{ type_name }}, args: RustBuffer.ByValue, outBuf: RustBufferByReference): Int {
    {%- endif %}
        {%- if meth.arguments().len() > 0 %}
        val argsBuf = args.asByteBuffer() ?: throw InternalException("No ByteBuffer in RustBuffer; this is a Uniffi bug")
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
            outBuf.setValue({{ return_type|ffi_converter_name }}.lowerIntoRustBuffer(returnValue))
            return 1
        }
        {%- when None %}
        fun makeCall() : Int {
            kotlinCallbackInterface.{{ meth.name()|fn_name }}(
                {%- for arg in meth.arguments() %}
                {{ arg|read_fn }}(argsBuf)
                {%- if !loop.last %}, {% endif %}
                {%- endfor %}
            )
            return 1
        }
        {%- endmatch %}

        {%- match meth.throws_type() %}
        {%- when None %}
        fun makeCallAndHandleError() : Int = makeCall()
        {%- when Some(error_type) %}
        fun makeCallAndHandleError()  : Int = try {
            makeCall()
        } catch (e: {{ error_type|type_name }}) {
            // Expected error, serialize it into outBuf
            outBuf.setValue({{ error_type|ffi_converter_name }}.lowerIntoRustBuffer(e))
            -2
        }
        {%- endmatch %}

        {%- if new_callback_interface_abi %}
        return makeCallAndHandleError()
        {%- else %}
        try {
            return makeCallAndHandleError()
        } finally {
            RustBuffer.free(args)
        }
        {%- endif %}
    }
    {% endfor %}
}

// The ffiConverter which transforms the Callbacks in to Handles to pass to Rust.
public object {{ ffi_converter_name }}: FfiConverterCallbackInterface<{{ type_name }}>(
    foreignCallback = {{ foreign_callback }}()
) {
    override fun register(lib: _UniFFILib) {
        rustCall() { status ->
            {%- if new_callback_interface_abi %}
            lib.{{ cbi.ffi_init_callback2().name() }}(this.foreignCallback, status)
            {%- else %}
            lib.{{ cbi.ffi_init_callback().name() }}(this.foreignCallback, status)
            {%- endif %}
        }
    }
}
