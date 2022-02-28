{%- match config %}
{%- when None %}
{#- No config, just forward all methods to our builtin type #}
fileprivate typealias FfiConverterType{{ name }} = {{ builtin|ffi_converter_name }}

{%- when Some with (config) %}
fileprivate struct FfiConverterType{{ name }} {
    {#- Custom type config supplied, use it to convert the builtin type #}

    static func read(from buf: Reader) throws -> {{ self.type_name(config) }} {
        let builtinValue = try {{ builtin|read_fn }}(from: buf)
        return {{ config.into_custom.render("builtinValue") }}
    }

    static func write(_ value: {{ self.type_name(config) }}, into buf: Writer) {
        let builtinValue = {{ config.from_custom.render("value") }}
        return {{ builtin|write_fn }}(builtinValue, into: buf)
    }

    static func lift(_ value: {{ self.builtin_ffi_type().borrow()|type_ffi_lowered }}) throws -> {{ self.type_name(config) }} {
        let builtinValue = try {{ builtin|lift_fn }}(value)
        return {{ config.into_custom.render("builtinValue") }}
    }

    static func lower(_ value: {{ self.type_name(config) }}) -> {{ self.builtin_ffi_type().borrow()|type_ffi_lowered }} {
        let builtinValue = {{ config.from_custom.render("value") }}
        return {{ builtin|lower_fn }}(builtinValue)
    }

}
{%- endmatch %}

