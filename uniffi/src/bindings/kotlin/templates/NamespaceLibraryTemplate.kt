// This is how we find and load the dynamic library provided by the component.
// For now we just look it up by name.
//
// XXX TODO: This will probably grow some magic for resolving megazording in future.
// E.g. we might start by looking for the named component in `libuniffi.so` and if
// that fails, fall back to loading it separately from `lib${componentName}.so`.

inline fun <reified Lib : Library> loadIndirect(
    componentName: String
): Lib {
    return Native.load<Lib>("uniffi_${componentName}", Lib::class.java)
}

// A JNA Library to expose the extern-C FFI definitions.
// This is an implementation detail which will be called internally by the public API.

internal interface _UniFFILib : Library {
    companion object {
        internal var INSTANCE: _UniFFILib = loadIndirect(componentName = "{{ ci.namespace() }}")
    }

    {% for func in ci.iter_ffi_function_definitions() -%}
        fun {{ func.name() }}(
        {%- for arg in func.arguments() %}
            {{ arg.name() }}: {{ arg.type_()|type_c }}{% if loop.last %}{% else %},{% endif %}
        {%- endfor %}
        // TODO: When we implement error handling, there will be an out error param here.
        ): {%- match func.return_type() -%}{%- when Some with (type_) %}{{ type_|ret_type_c }}{% when None %}Unit{% endmatch %}
    {% endfor -%}
}
