{%- call swift::docstring(e, 0) %}
public enum {{ type_name }}: Swift.Error, {{ config.conformance_list_for_error(e, contains_object_references) }} {

    {% if e.is_flat() %}
    {% for variant in e.variants() %}
    {%- call swift::docstring(variant, 4) %}
    case {{ variant.name()|class_name }}(message: String)
    {% endfor %}

    {%- else %}
    {% for variant in e.variants() %}
    {%- call swift::docstring(variant, 4) %}
    case {{ variant.name()|class_name }}{% if variant.fields().len() > 0 %}(
        {%- call swift::field_list_decl(variant, variant.has_nameless_fields()) %}
    ){% endif -%}
    {% endfor %}

    {%- endif %}

    {% call swift::uniffi_trait_impls(e.uniffi_trait_methods()) %}

    {% if !config.omit_localized_error_conformance() %}
    public var errorDescription: String? {
        String(reflecting: self)
    }
    {% endif %}
}


#if swift(>=5.8)
@_documentation(visibility: private)
#endif
public struct {{ ffi_converter_name }}: FfiConverterRustBuffer {
    typealias SwiftType = {{ type_name }}

    public static func read(from buf: inout (data: Data, offset: Data.Index)) throws -> {{ type_name }} {
        let variant: Int32 = try readInt(&buf)
        switch variant {

        {% if e.is_flat() %}

        {% for variant in e.variants() %}
        case {{ loop.index }}: return .{{ variant.name()|class_name }}(
            message: try {{ Type::String.borrow()|read_fn }}(from: &buf)
        )
        {% endfor %}

        {% else %}

        {% for variant in e.variants() %}
        case {{ loop.index }}: return .{{ variant.name()|error_variant_swift_quoted }}{% if variant.has_fields() %}(
            {% for field in variant.fields() -%}
            {%-     if variant.has_nameless_fields() -%}
            try {{ field|read_fn }}(from: &buf)
            {%-     else -%}
            {{ field.name()|var_name }}: try {{ field|read_fn }}(from: &buf)
            {%-     endif -%}
            {%- if !loop.last %}, {% endif %}
            {% endfor -%}
        ){% endif -%}
        {% endfor %}

         {% endif -%}
        default: throw UniffiInternalError.unexpectedEnumCase
        }
    }

    public static func write(_ value: {{ type_name }}, into buf: inout [UInt8]) {
        switch value {

        {% if e.is_flat() %}

        {% for variant in e.variants() %}
        case .{{ variant.name()|class_name }}(_ /* message is ignored*/):
            writeInt(&buf, Int32({{ loop.index }}))
        {%- endfor %}

        {% else %}

        {% for variant in e.variants() %}
        {% if variant.has_fields() %}
        case let .{{ variant.name()|error_variant_swift_quoted }}({% for field in variant.fields() %}{%- call swift::field_name(field, loop.index) -%}{%- if loop.last -%}{%- else -%},{%- endif -%}{% endfor %}):
            writeInt(&buf, Int32({{ loop.index }}))
            {% for field in variant.fields() -%}
            {{ field|write_fn }}({% call swift::field_name(field, loop.index) %}, into: &buf)
            {% endfor -%}
        {% else %}
        case .{{ variant.name()|class_name }}:
            writeInt(&buf, Int32({{ loop.index }}))
        {% endif %}
        {%- endfor %}

        {%- endif %}
        }
    }
}

{#
We always write these public functions just in case the error is used as
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
