@Synchronized
fun findLibraryName(componentName: String): String {
    val libOverride = System.getProperty("uniffi.component.${componentName}.libraryOverride")
    if (libOverride != null) {
        return libOverride
    }
    return "uniffi_${componentName}"
}

inline fun <reified Lib : Library> loadIndirect(
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
            {% let callback_interfaces = ci.iter_callback_interface_definitions() %}
            {%- if !callback_interfaces.is_empty() -%}
            .also { lib: _UniFFILib ->
                {% for cb in callback_interfaces -%}
                CallbackInterface{{ cb.name()|class_name_kt }}Internals.register(lib)
                {% endfor -%}
            }
            {% endif %}
        }
    }

    {% for func in ci.iter_ffi_function_definitions() -%}
    fun {{ func.name() }}(
        {%- call kt::arg_list_ffi_decl(func) %}
    ){%- match func.return_type() -%}{%- when Some with (type_) %}: {{ type_|type_ffi }}{% when None %}: Unit{% endmatch %}

    {% endfor %}
}
