object FfiConverterType{{ name }} {
    {%- match config %}
    {%- when None %}
    {#- No custom type config, just forward all methods to our builtin type #}
    fun write(value: {{ builtin|type_name }}, buf: RustBufferBuilder) {
        return {{ "value"|write_var("buf", builtin) }}
    }

    fun read(buf: ByteBuffer): {{ builtin|type_name }} {
        return {{ "buf"|read_var(builtin) }}
    }

    fun lift(value: {{ self.builtin_ffi_type()|ffi_type_name }}): {{ builtin|type_name }} {
        return {{ "value"|lift_var(builtin) }}
    }

    fun lower(value: {{ builtin|type_name }}): {{ self.builtin_ffi_type()|ffi_type_name }} {
        return {{ "value"|lower_var(builtin) }}
    }

    {%- when Some with (config) %}
    {#- Custom type config supplied, use it to convert the builtin type #}
    fun write(value: {{ self.type_name(config) }}, buf: RustBufferBuilder) {
        val builtinValue = {{ config.from_custom.render("value") }}
        return {{ "builtinValue"|write_var("buf", builtin) }}
    }

    fun read(buf: ByteBuffer): {{ self.type_name(config) }} {
        val builtinValue = ({{ "buf"|read_var(builtin) }})
        return {{ config.into_custom.render("builtinValue") }}
    }

    fun lift(value: {{ self.builtin_ffi_type()|ffi_type_name }}): {{ self.type_name(config) }} {
        val builtinValue = ({{ "value"|lift_var(builtin) }})
        return {{ config.into_custom.render("builtinValue") }}
    }

    fun lower(value: {{ self.type_name(config) }}): {{ self.builtin_ffi_type()|ffi_type_name }} {
        val builtinValue = {{ config.from_custom.render("value") }}
        return {{ "builtinValue"|lower_var(builtin) }}
    }
    {%- endmatch %}
}
