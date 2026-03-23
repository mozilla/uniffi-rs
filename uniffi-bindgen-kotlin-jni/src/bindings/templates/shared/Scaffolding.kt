object Scaffolding {
    {%- for package in root.packages %}
    {%- for func in package.functions %}
    @JvmStatic external fun {{ func.jni_method_name }}()
    {%- endfor %}
    {%- endfor %}

    // access `uniffiLibrary` to make sure the cdylib is loaded
    init {
        System.loadLibrary("{{ cdylib }}")
    }
}
