{%- let rec = ci.get_record_definition(name).unwrap() %}
{%- let uniffi_trait_methods = rec.uniffi_trait_methods() %}
{%- call swift::docstring(rec, 0) %}
{%- if config.record_has_conformances(rec, contains_object_references) %}
public struct {{ type_name }}: {{ config.conformance_list_for_record(rec, contains_object_references) }} {
{%- else %}
public struct {{ type_name }} {
{%- endif %}
    {%- for field in rec.fields() %}
    {%- call swift::docstring(field, 4) %}
    public {% if config.generate_immutable_records() %}let{% else %}var{% endif %} {{ field.name()|var_name }}: {{ field|type_name }}
    {%- endfor %}

    // Default memberwise initializers are never public by default, so we
    // declare one manually.
    public init({% call swift::field_list_decl(rec, false) %}) {
        {%- for field in rec.fields() %}
        self.{{ field.name()|var_name }} = {{ field.name()|var_name }}
        {%- endfor %}
    }

    {% for meth in rec.methods() -%}
    {%- call swift::func_decl("public func", meth, 4) %}
    {% endfor %}

    {% call swift::uniffi_trait_impls(uniffi_trait_methods) %}
}

#if compiler(>=6)
extension {{ type_name }}: Sendable {}
#endif

{%- for t in rec.trait_impls() %}
extension {{ type_name }}: {{ self::trait_protocol_name(ci, t.trait_ty)? }} {}
{% endfor %}

#if swift(>=5.8)
@_documentation(visibility: private)
#endif
public struct {{ ffi_converter_name }}: FfiConverterRustBuffer {
    public static func read(from buf: inout (data: Data, offset: Data.Index)) throws -> {{ type_name }} {
        return {%- if rec.has_fields() %}
            try {{ type_name }}(
            {%- for field in rec.fields() %}
                {{ field.name()|arg_name }}: {{ field|read_fn }}(from: &buf)
                {%- if !loop.last %}, {% endif %}
            {%- endfor %}
        )
        {%- else %}
            {{ type_name }}()
        {%- endif %}
    }

    public static func write(_ value: {{ type_name }}, into buf: inout [UInt8]) {
        {%- for field in rec.fields() %}
        {{ field|write_fn }}(value.{{ field.name()|var_name }}, into: &buf)
        {%- endfor %}
    }
}

{#
We always write these public functions just in case the struct is used as
an external type by another crate.
#}
#if swift(>=5.8)
@_documentation(visibility: private)
#endif
public func {{ ffi_converter_name }}_lift(_ buf: RustBuffer) throws -> {{ type_name }} {
    return try {{ ffi_converter_name }}.lift(buf)
}

#if swift(>=5.8)
@_documentation(visibility: private)
#endif
public func {{ ffi_converter_name }}_lower(_ value: {{ type_name }}) -> RustBuffer {
    return {{ ffi_converter_name }}.lower(value)
}
