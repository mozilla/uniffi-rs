internal interface BindgenRendererFFILib : com.sun.jna.Library {
    {%- for name, func in functions %}
    fun {{ name }}({% for arg in func.args %}{{ arg.name }}: {{ arg.type|ffi_type }}{% if not loop.last %}, {% endif %}{% endfor %}){% if func.return_type %}: {{ func.return_type|ffi_type }}{% endif %}
    {%- endfor %}

    companion object {
        internal val ffiLib: BindgenRendererFFILib by lazy {
            com.sun.jna.Native.load("{{ name }}", BindgenRendererFFILib::class.java)
        }

        {%- for name, func in functions %}
        fun {{ name|to_lower_camel_case }}({{ func.args|comma_join }}){% if func.return_type %}: {{ func.return_type }}{% endif %} {
            {%- set return_var = temp_var_name() %}
            {%- for arg in func.args %}
            {%- if arg.type is struct_reference %}
            {{ arg.name }}.write()
            {%- endif %}
            {%- endfor %}

            val {{ return_var }} = this.ffiLib.{{ name }}(
                {%- for arg in func.args %}
                {%- set converter = arg.type|to_ffi_converter %}
                {%- if converter %}
                {{ arg.name }}.{{ converter }}(),
                {%- elif arg.type is struct_reference %}
                {{ arg.name }}.getPointer(),
                {%- else %}
                {{ arg.name }},
                {%- endif %}
                {%- endfor %}
            )

            {%- for arg in func.args %}
            {%- if arg.type is struct_reference and arg.type.mutable %}
            {{ arg.name }}.read()
            {%- endif %}
            {%- endfor %}

            {%- if func.return_type %}
            {%- set converter = func.return_type|from_ffi_converter %}
            {%- if converter %}
            return {{ return_var }}.{{ converter }}()
            {%- else %}
            return {{ return_var }}
            {%- endif %}
            {%- endif %}
        }
        {%- endfor %}
    }
}
