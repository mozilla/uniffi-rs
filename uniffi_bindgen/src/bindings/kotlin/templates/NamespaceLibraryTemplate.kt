@Synchronized
private fun findLibraryName(componentName: String): String {
    val libOverride = System.getProperty("uniffi.component.$componentName.libraryOverride")
    if (libOverride != null) {
        return libOverride
    }
    return "{{ config.cdylib_name() }}"
}

private inline fun <reified Lib : Library> loadIndirect(
    componentName: String
): Lib {
    return Native.load<Lib>(findLibraryName(componentName), Lib::class.java)
}

// A JNA Library to expose the extern-C FFI definitions.
// This is an implementation detail which will be called internally by the public API.

internal interface _UniFFILib : Library {
    companion object {
        internal val INSTANCE: _UniFFILib by lazy {
            loadIndirect<_UniFFILib>(componentName = "{{ ci.namespace() }}")
            {% let initialization_fns = self.initialization_fns() %}
            {%- if !initialization_fns.is_empty() -%}
            .also { lib: _UniFFILib ->
                {% for fn in initialization_fns -%}
                {{ fn }}(lib)
                {% endfor -%}
            }
            {% endif %}
        }
    }

    {% for func in ci.iter_ffi_function_definitions() -%}
    {%- if func.is_async() %}
    fun {{ func.name() }}(
        {%- call kt::arg_list_ffi_decl(func) %}
    ): RustFuture
    
    fun {{ func.name() }}_poll(
        rustFuture: RustFuture,
        waker: RustFutureWaker,
        wakerEnv: Pointer?,
        polledResult: {% match func.return_type() %}{% when Some with (return_type) %}{{ return_type|type_ffi_lowered }}{% when None %}Int{% endmatch %}ByReference,
        _uniffi_out_err: RustCallStatus
    ): Boolean
    
    fun {{ func.name() }}_drop(
        `rust_future`: RustFuture,
        _uniffi_out_err: RustCallStatus
    )
    {%- else %}
    fun {{ func.name() }}(
        {%- call kt::arg_list_ffi_decl(func) %}
    ): {% match func.return_type() %}{% when Some with (return_type) %}{{ return_type.borrow()|ffi_type_name }}{% when None %}Unit{% endmatch %}
    {%- endif -%}

    {% endfor %}
}
