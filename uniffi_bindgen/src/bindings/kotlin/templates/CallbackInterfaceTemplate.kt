{% import "macros.kt" as kt %}
{%- let cbi = self.inner() %}
{%- let type_name = cbi|type_name %}
{%- let canonical_type_name = cbi|canonical_name %}
{%- let ffi_converter = cbi|ffi_converter_name %}
{%- let foreign_callback = format!("ForeignCallback{}", canonical_type_name) %}

// Declaration and FfiConverters for {{ type_name }} Callback Interface

{% if internalize %}internal {% endif %}interface {{ type_name }} {
    {% for meth in cbi.methods() -%}
    fun {{ meth.name()|fn_name }}({% call kt::arg_list_decl(meth) %})
    {%- match meth.return_type() -%}
    {%- when Some with (return_type) %}: {{ return_type|type_name -}}
    {%- else -%}
    {%- endmatch %}
    {% endfor %}
}

// The ForeignCallback that is passed to Rust.
internal class {{ foreign_callback }} : ForeignCallback {
    @Suppress("TooGenericExceptionCaught")
    override fun invoke(handle: Handle, method: Int, args: RustBuffer.ByValue, outBuf: RustBufferByReference): Int {
        val cb = {{ ffi_converter }}.lift(handle)
        return when (method) {
            IDX_CALLBACK_FREE -> {
                {{ ffi_converter }}.drop(handle)
                // No return value.
                // See docs of ForeignCallback in `uniffi/src/ffi/foreigncallbacks.rs`
                0
            }
            {% for meth in cbi.methods() -%}
            {% let method_name = format!("invoke_{}", meth.name())|fn_name -%}
            {{ loop.index }} -> {
                val buffer = this.{{ method_name }}(cb, args)
                outBuf.setValue(buffer)
                // Value written to out buffer.
                // See docs of ForeignCallback in `uniffi/src/ffi/foreigncallbacks.rs`
                1
            }
            {% endfor %}
            // This should never happen, because an out of bounds method index won't
            // ever be used. Once we can catch errors, we should return an InternalException.
            // https://github.com/mozilla/uniffi-rs/issues/351
            else -> {
                // An unexpected error happened.
                // See docs of ForeignCallback in `uniffi/src/ffi/foreigncallbacks.rs`
                -1
            }
        }
    }

    {% for meth in cbi.methods() -%}
    {% let method_name = format!("invoke_{}", meth.name())|fn_name %}
    private fun {{ method_name }}(kotlinCallbackInterface: {{ type_name }}, args: RustBuffer.ByValue): RustBuffer.ByValue =
        try {
        {#- Unpacking args from the RustBuffer #}
            {%- if meth.arguments().len() != 0 -%}
            {#- Calling the concrete callback object #}
            val buf = args.asByteBuffer() ?: throw InternalException("No ByteBuffer in RustBuffer; this is a Uniffi bug")
            kotlinCallbackInterface.{{ meth.name()|fn_name }}(
                    {% for arg in meth.arguments() -%}
                    {{ arg|read_fn }}(buf)
                    {%- if !loop.last %}, {% endif %}
                    {% endfor -%}
                )
            {% else %}
            kotlinCallbackInterface.{{ meth.name()|fn_name }}()
            {% endif -%}

        {#- Packing up the return value into a RustBuffer #}
                {%- match meth.return_type() -%}
                {%- when Some with (return_type) -%}
                .let {
                    {{ return_type|ffi_converter_name }}.lowerIntoRustBuffer(it)
                }
                {%- else -%}
                .let { RustBuffer.ByValue() }
                {% endmatch -%}
                // TODO catch errors and report them back to Rust.
                // https://github.com/mozilla/uniffi-rs/issues/351
        } finally {
            RustBuffer.free(args)
        }

    {% endfor %}
}

// The ffiConverter which transforms the Callbacks in to Handles to pass to Rust.
{% if internalize %}internal {% endif %}object {{ ffi_converter }}: FfiConverterCallbackInterface<{{ type_name }}>(
    foreignCallback = {{ foreign_callback }}()
) {
    override fun register(lib: _UniFFILib) {
        rustCall() { status ->
            lib.{{ cbi.ffi_init_callback().name() }}(this.foreignCallback, status)
        }
    }
}
