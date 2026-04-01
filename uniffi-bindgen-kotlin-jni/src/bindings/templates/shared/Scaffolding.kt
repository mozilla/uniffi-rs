object Scaffolding {
    {%- for (jni_method_name, callable) in root.jni_methods() %}
    @JvmStatic external fun {{ jni_method_name }}(
        {%- for ffi_arg in callable.ffi_arguments() %}
        {{ ffi_arg.name_kt() }}: {{ ffi_arg.ty.type_kt() }},
        {%- endfor %}
    )
    {%- match callable.return_ffi %}
    {%- when ReturnFfi::Primitive { ffi_type, .. } %} : {{ ffi_type.type_kt() }}
    {%- when ReturnFfi::Deconstruct { type_node, .. } %} : {{ type_node.type_kt }}
    {%- when ReturnFfi::Void %}
    {%- endmatch %}
    {%- endfor %}

    init {
        System.loadLibrary("{{ cdylib }}")
    }
}
