{%- let e = ci.get_error_definition(name).unwrap() %}
class {{ type_name }} implements Exception {
    {% for variant in e.variants() %}
    // Simple error enums only carry a message
    static const int {{ variant.name()|class_name }} = {{ loop.index }};
    {% endfor %}
    final int errorCode;

    const {{ type_name }}(this.errorCode);

    @override
    String toString() {
        switch (errorCode) {
            {% for variant in e.variants() %}
            case {{ variant.name()|class_name }}:
                return "{{ type_name }}::{{ variant.name()|class_name }}";
            {% endfor %}
            default:
                return "{{ type_name }}::UnknownError: $errorCode";
        }
    }
}

class {{ ffi_converter_name }} {
    static {{ type_name }} read(Uint8List buf) {
        print("1234567890");
        var bytes = ByteData.view(buf.buffer);
        int errorCode = bytes.getInt32(0);
        print("error code - $errorCode");

        switch (errorCode) {
        {% for variant in e.variants() %}
        case {{ type_name }}.{{ variant.name()|class_name }}:
            return {{ type_name }}({{ type_name }}.{{ variant.name()|class_name }});
        {% endfor %}
        default:
            throw UniffiInternalError.unexpectedEnumCase;
        }
    }

    static void write({{ type_name }} value, Uint8List buf) {
        switch (value.errorCode) {
        {% for variant in e.variants() %}
        case {{ type_name }}.{{ variant.name()|class_name }}: {
            var bytes = ByteData.view(buf.buffer);
            bytes.setUint32(0, {{ type_name }}.{{ variant.name()|class_name }});
        }
        break;
        {%- endfor %}
        }
    }
}
