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

// Define FFI callback types
{%- for def in ci.ffi_definitions() %}
{%- match def %}
{%- when FfiDefinition::CallbackFunction(callback) %}
internal interface {{ callback.name()|ffi_callback_name }} : com.sun.jna.Callback {
    fun callback(
        {%- for arg in callback.arguments() -%}
        {{ arg.name().borrow()|var_name }}: {{ arg.type_().borrow()|ffi_type_name_by_value(ci) }},
        {%- endfor -%}
        {%- if callback.has_rust_call_status_arg() -%}
        uniffiCallStatus: UniffiRustCallStatus,
        {%- endif -%}
    )
    {%- if let Some(return_type) = callback.return_type() %}
    : {{ return_type|ffi_type_name_by_value(ci) }}
    {%- endif %}
}
{%- when FfiDefinition::Struct(ffi_struct) %}
@Structure.FieldOrder({% for field in ffi_struct.fields() %}"{{ field.name()|var_name_raw }}"{% if !loop.last %}, {% endif %}{% endfor %})
internal open class {{ ffi_struct.name()|ffi_struct_name }}(
    {%- for field in ffi_struct.fields() %}
    @JvmField internal var {{ field.name()|var_name }}: {{ field.type_().borrow()|ffi_type_name_for_ffi_struct(ci) }} = {{ field.type_()|ffi_default_value }},
    {%- endfor %}
) : Structure() {
    class UniffiByValue(
        {%- for field in ffi_struct.fields() %}
        {{ field.name()|var_name }}: {{ field.type_().borrow()|ffi_type_name_for_ffi_struct(ci) }} = {{ field.type_()|ffi_default_value }},
        {%- endfor %}
    ): {{ ffi_struct.name()|ffi_struct_name }}({%- for field in ffi_struct.fields() %}{{ field.name()|var_name }}, {%- endfor %}), Structure.ByValue

   internal fun uniffiSetValue(other: {{ ffi_struct.name()|ffi_struct_name }}) {
        {%- for field in ffi_struct.fields() %}
        {{ field.name()|var_name }} = other.{{ field.name()|var_name }}
        {%- endfor %}
    }

}
{%- when FfiDefinition::Function(_) %}
{# functions are handled below #}
{%- endmatch %}
{%- endfor %}


{%- macro decl_kotlin_functions(func_list) -%}
{% for func in func_list -%}
fun {{ func.name() }}(
    {%- call kt::arg_list_ffi_decl(func) %}
): {% match func.return_type() %}{% when Some with (return_type) %}{{ return_type.borrow()|ffi_type_name_by_value(ci) }}{% when None %}Unit{% endmatch %}
{% endfor %}
{%- endmacro %}

// For large crates we prevent `MethodTooLargeException` (see #2340)
// N.B. the name of the extension is very misleading, since it is 
// rather `InterfaceTooLargeException`, caused by too many methods 
// in the interface for large crates.
//
// By splitting the otherwise huge interface into two parts
// * UniffiLib 
// * IntegrityCheckingUniffiLib (this)
// we allow for ~2x as many methods in the UniffiLib interface.
// 
// The `ffi_uniffi_contract_version` method and all checksum methods are put 
// into `IntegrityCheckingUniffiLib` and these methods are called only once,
// when the library is loaded.
internal interface IntegrityCheckingUniffiLib : Library {
    // Integrity check functions only
    {# newline below wanted #}

{%- call decl_kotlin_functions(ci.iter_ffi_function_integrity_checks()) %}
}

// A JNA Library to expose the extern-C FFI definitions.
// This is an implementation detail which will be called internally by the public API.
internal interface UniffiLib : Library {
    companion object {
        internal val INSTANCE: UniffiLib by lazy {
            val componentName = "{{ ci.namespace() }}"
            // For large crates we prevent `MethodTooLargeException` (see #2340)
            // N.B. the name of the extension is very misleading, since it is 
            // rather `InterfaceTooLargeException`, caused by too many methods 
            // in the interface for large crates.
            //
            // By splitting the otherwise huge interface into two parts
            // * UniffiLib (this)
            // * IntegrityCheckingUniffiLib
            // And all checksum methods are put into `IntegrityCheckingUniffiLib`
            // we allow for ~2x as many methods in the UniffiLib interface.
            // 
            // Thus we first load the library with `loadIndirect` as `IntegrityCheckingUniffiLib`
            // so that we can call `uniffiCheckApiChecksums`...
            loadIndirect<IntegrityCheckingUniffiLib>(componentName)
                .also { lib: IntegrityCheckingUniffiLib ->
                    uniffiCheckContractApiVersion(lib)
                    uniffiCheckApiChecksums(lib)
                }
            // ... and then we load the library as `UniffiLib`
            // N.B. we cannot use `loadIndirect` once and then try to cast it to `UniffiLib`
            // => results in `java.lang.ClassCastException: com.sun.proxy.$Proxy cannot be cast to ...`
            // error. So we must call `loadIndirect` twice. For crates large enough
            // to trigger this issue, the performance impact is negligible, running on
            // a macOS M1 machine the `loadIndirect` call takes ~50ms.
            val lib = loadIndirect<UniffiLib>(componentName)
            // No need to check the contract version and checksums, since 
            // we already did that with `IntegrityCheckingUniffiLib` above.
            {% for fn in self.initialization_fns(ci) -%}
            {{ fn }}
            {% endfor -%}
            // Loading of library with integrity check done.
            lib
        }
        {% if ci.contains_object_types() %}
        // The Cleaner for the whole library
        internal val CLEANER: UniffiCleaner by lazy {
            UniffiCleaner.create()
        }
        {%- endif %}
    }

    // FFI functions
    {# newline below before call decl_kotlin_functions is needed #}

    {%- call decl_kotlin_functions(ci.iter_ffi_function_definitions_excluding_integrity_checks()) %}
}

private fun uniffiCheckContractApiVersion(lib: IntegrityCheckingUniffiLib) {
    // Get the bindings contract version from our ComponentInterface
    val bindings_contract_version = {{ ci.uniffi_contract_version() }}
    // Get the scaffolding contract version by calling the into the dylib
    val scaffolding_contract_version = lib.{{ ci.ffi_uniffi_contract_version().name() }}()
    if (bindings_contract_version != scaffolding_contract_version) {
        throw RuntimeException("UniFFI contract version mismatch: try cleaning and rebuilding your project")
    }
}

@Suppress("UNUSED_PARAMETER")
private fun uniffiCheckApiChecksums(lib: IntegrityCheckingUniffiLib) {
    {%- for (name, expected_checksum) in ci.iter_checksums() %}
    if (lib.{{ name }}() != {{ expected_checksum }}.toShort()) {
        throw RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project")
    }
    {%- endfor %}
}

/**
 * @suppress
 */
public fun uniffiEnsureInitialized() {
    UniffiLib.INSTANCE
}
