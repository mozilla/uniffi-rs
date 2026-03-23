object Scaffolding {
    {%- for (jni_method_name, callable) in root.jni_methods() %}
    @JvmStatic external fun {{ jni_method_name }}()
    {%- endfor %}

    init {
        System.loadLibrary("{{ cdylib }}")
    }
}
