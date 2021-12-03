fileprivate struct FfiConverterType{{ name }} {
    {%- match config %}
    {%- when None %}
    {#- No config, just forward all methods to our builtin type #}
    fileprivate static func read(_ buf: Reader) throws -> {{ builtin|type_name }} {
        return try {{ "buf"|read_var(builtin) }}
    }

    fileprivate static func write(_ value: {{ builtin|type_name }}, _ buf: Writer) {
        return {{ "value"|write_var("buf", builtin) }}
    }

    fileprivate static func lift(_ value: {{ self.builtin_ffi_type()|type_ffi_lowered }}) throws -> {{ builtin|type_name }} {
        return try {{ "value"|lift_var(builtin) }}
    }

    fileprivate static func lower(_ value: {{ builtin|type_name }}) -> {{ self.builtin_ffi_type()|type_ffi_lowered }} {
        return {{ "value"|lower_var(builtin) }}
    }

    {%- when Some with (config) %}
    {#- Custom type config supplied, use it to convert the builtin type #}

    fileprivate static func read(_ buf: Reader) throws -> {{ self.type_name(config) }} {
        let builtinValue = try {{ "buf"|read_var(builtin) }}
        return {{ config.into_custom.render("builtinValue") }}
    }

    fileprivate static func write(_ value: {{ self.type_name(config) }}, _ buf: Writer) {
        let builtinValue = {{ config.from_custom.render("value") }}
        return {{ "builtinValue"|write_var("buf", builtin) }}
    }

    fileprivate static func lift(_ value: {{ self.builtin_ffi_type()|type_ffi_lowered }}) throws -> {{ self.type_name(config) }} {
        let builtinValue = try {{ "value"|lift_var(builtin) }}
        return {{ config.into_custom.render("builtinValue") }}
    }

    fileprivate static func lower(_ value: {{ self.type_name(config) }}) -> {{ self.builtin_ffi_type()|type_ffi_lowered }} {
        let builtinValue = {{ config.from_custom.render("value") }}
        return {{ "builtinValue"|lower_var(builtin) }}
    }

    {%- endmatch %}
}

