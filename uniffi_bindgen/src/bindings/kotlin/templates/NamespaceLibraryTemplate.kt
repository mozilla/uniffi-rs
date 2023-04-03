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
            .also { lib: _UniFFILib ->
                uniffiCheckContractApiVersion(lib)
                uniffiCheckApiChecksums(lib)
                {% for fn in self.initialization_fns() -%}
                {{ fn }}(lib)
                {% endfor -%}
            }
        }

        {%- if ci.has_async_fns() %}
        internal val FUTURE_WAKER_ENVIRONMENTS: ConcurrentHashMap<Int, RustFutureWakerEnvironment<out Any>> by lazy {
            ConcurrentHashMap(8)
        }
        {%- endif %}
    }

    {% for func in ci.iter_ffi_function_definitions() -%}
    {%- if func.is_async() %}
    fun {{ func.name() }}(
        {%- call kt::arg_list_ffi_decl(func) %}
    ): RustFuture

    fun {{ func.name() }}_poll(
        rustFuture: RustFuture,
        waker: RustFutureWaker,
        wakerEnv: RustFutureWakerEnvironmentCStructure?,
        polledResult: {% match func.return_type() %}{% when Some with (return_type) %}{{ return_type|type_ffi_lowered }}{% when None %}Pointer{% endmatch %}ByReference,
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

private fun uniffiCheckContractApiVersion(lib: _UniFFILib) {
    // Get the bindings contract version from our ComponentInterface
    val bindings_contract_version = {{ ci.uniffi_contract_version() }}
    // Get the scaffolding contract version by calling the into the dylib
    val scaffolding_contract_version = lib.{{ ci.ffi_uniffi_contract_version().name() }}()
    if (bindings_contract_version != scaffolding_contract_version) {
        throw RuntimeException("UniFFI contract version mismatch: try cleaning and rebuilding your project")
    }
}

@Suppress("UNUSED_PARAMETER")
private fun uniffiCheckApiChecksums(lib: _UniFFILib) {
    {%- for (name, expected_checksum) in ci.iter_checksums() %}
    if (lib.{{ name }}() != {{ expected_checksum }}.toShort()) {
        throw RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project")
    }
    {%- endfor %}
}
