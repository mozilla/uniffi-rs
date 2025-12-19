@Synchronized
private fun findLibraryName(componentName: String): String {
    val libOverride = System.getProperty("uniffi.component.$componentName.libraryOverride")
    if (libOverride != null) {
        return libOverride
    }
    return "{{ config.cdylib_name() }}"
}

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
{#- functions are handled below #}
{%- endmatch %}
{%- endfor %}

// A JNA Library to expose the extern-C FFI definitions.
// This is an implementation detail which will be called internally by the public API.

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
// Note: above all written when we used JNA's `loadIndirect` etc.
// We now use JNA's "direct mapping" - unclear if same considerations apply exactly.
internal object IntegrityCheckingUniffiLib {
    init {
        Native.register(IntegrityCheckingUniffiLib::class.java, findLibraryName(componentName = "{{ ci.namespace() }}"))
        uniffiCheckContractApiVersion(this)
{%- if !config.omit_checksums %}
        uniffiCheckApiChecksums(this)
{%- endif %}
    }
    {%- for name in ci.pointer_ffi_integrity_function_names() %}
    external fun {{ name }}(uniffiFfiBuffer: Pointer)
    {%- endfor %}
}

internal object UniffiLib {
    {% if ci.contains_object_types() %}
    // The Cleaner for the whole library
    internal val CLEANER: UniffiCleaner by lazy {
        UniffiCleaner.create()
    }
    {% endif %}

    init {
        Native.register(UniffiLib::class.java, findLibraryName(componentName = "{{ ci.namespace() }}"))
        {% for fn_item in self.initialization_fns() -%}
        {{ fn_item }}
        {% endfor %}
    }

    {%- for name in ci.pointer_ffi_function_names() %}
    external fun {{ name }}(uniffiFfiBuffer: Pointer)
    {%- endfor %}
}

private fun uniffiCheckContractApiVersion(lib: IntegrityCheckingUniffiLib) {
    // Get the bindings contract version from our ComponentInterface
    val bindings_contract_version = {{ ci.uniffi_contract_version() }}
    // Get the scaffolding contract version by calling the into the dylib
    val ffiBuffer = Memory(8)
    lib.{{ ci.ffi_uniffi_contract_version().pointer_ffi_name() }}(ffiBuffer)
    val returnCursor = UniffiBufferCursor(ffiBuffer)
    if (bindings_contract_version != UniffiFfiSerializerInt.read(returnCursor)) {
        throw RuntimeException("UniFFI contract version mismatch: try cleaning and rebuilding your project")
    }
}

{%- if !config.omit_checksums %}
@Suppress("UNUSED_PARAMETER")
private fun uniffiCheckApiChecksums(lib: IntegrityCheckingUniffiLib) {
    {%- for (name, expected_checksum) in ci.pointer_ffi_iter_checksums() %}
    {%- if loop.first %}
    val ffiBuffer = Memory(8)
    var returnCursor: UniffiBufferCursor
    {%- endif %}
    lib.{{ name }}(ffiBuffer)
    returnCursor = UniffiBufferCursor(ffiBuffer)
    if (UniffiFfiSerializerShort.read(returnCursor) != {{ expected_checksum }}.toShort()) {
        throw RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project")
    }
    {%- endfor %}
}
{%- endif %}

/**
 * @suppress
 */
public fun uniffiEnsureInitialized() {
    IntegrityCheckingUniffiLib
    // UniffiLib() initialized as objects are used, but we still need to explicitly
    // reference it so initialization across crates works as expected.
    UniffiLib
}
