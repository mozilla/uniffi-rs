// Note that we don't yet support `indirect` for enums.
// See https://github.com/mozilla/uniffi-rs/issues/396 for further discussion.
{%- call swift::docstring(e, 0) %}
{%- let uniffi_trait_methods = e.uniffi_trait_methods() %}
{% match e.variant_discr_type() %}
{% when None %}
{%- if config.enum_has_conformances(e, contains_object_references) %}
public enum {{ type_name }}: {{ config.conformance_list_for_enum(e, contains_object_references) }} {
{%- else %}
public enum {{ type_name }} {
{%- endif %}
    {% for variant in e.variants() %}
    {%- call swift::docstring(variant, 4) %}
    case {{ variant.name()|enum_variant_swift_quoted }}{% if variant.fields().len() > 0 %}(
        {%- call swift::field_list_decl(variant, variant.has_nameless_fields()) %}
    ){% endif -%}
    {% endfor %}
{% when Some(variant_discr_type) %}
public enum {{ type_name }}: {{ variant_discr_type|type_name }}, {{ config.conformance_list_for_enum(e, contains_object_references) }} {
    {% for variant in e.variants() %}
    {%- call swift::docstring(variant, 4) %}
    case {{ variant.name()|enum_variant_swift_quoted }} = {{ e|variant_discr_literal(loop.index0) }}{% if variant.fields().len() > 0 %}(
        {%- call swift::field_list_decl(variant, variant.has_nameless_fields()) %}
    ){% endif -%}
    {% endfor %}
{% endmatch %}

{% for meth in e.methods() -%}
{%- call swift::func_decl("public func", meth, 4) %}
{% endfor %}

{% call swift::uniffi_trait_impls(uniffi_trait_methods) %}
}

#if compiler(>=6)
extension {{ type_name }}: Sendable {}
#endif

#if swift(>=5.8)
@_documentation(visibility: private)
#endif
public struct {{ ffi_converter_name }}: FfiConverterRustBuffer {
    typealias SwiftType = {{ type_name }}

    public static func read(from buf: inout (data: Data, offset: Data.Index)) throws -> {{ type_name }} {
        let variant: Int32 = try readInt(&buf)
        switch variant {
        {% for variant in e.variants() %}
        case {{ loop.index }}: return .{{ variant.name()|enum_variant_swift_quoted }}{% if variant.has_fields() %}(
            {%- for field in variant.fields() %}
            {%- if variant.has_nameless_fields() -%}
            try {{ field|read_fn }}(from: &buf)
            {%- else -%}
            {{ field.name()|arg_name }}: try {{ field|read_fn }}(from: &buf)
            {%- endif -%}
            {%- if !loop.last %}, {% endif %}
            {%- endfor %}
        ){%- endif %}
        {% endfor %}
        default: throw UniffiInternalError.unexpectedEnumCase
        }
    }

    public static func write(_ value: {{ type_name }}, into buf: inout [UInt8]) {
        switch value {
        {% for variant in e.variants() %}
        {% if variant.has_fields() %}
        case let .{{ variant.name()|enum_variant_swift_quoted }}({% for field in variant.fields() %}{%- call swift::field_name(field, loop.index) -%}{%- if loop.last -%}{%- else -%},{%- endif -%}{% endfor %}):
            writeInt(&buf, Int32({{ loop.index }}))
            {% for field in variant.fields() -%}
            {{ field|write_fn }}({% call swift::field_name(field, loop.index) %}, into: &buf)
            {% endfor -%}
        {% else %}
        case .{{ variant.name()|enum_variant_swift_quoted }}:
            writeInt(&buf, Int32({{ loop.index }}))
        {% endif %}
        {%- endfor %}
        }
    }
}

{#
We always write these public functions just in case the enum is used as
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
