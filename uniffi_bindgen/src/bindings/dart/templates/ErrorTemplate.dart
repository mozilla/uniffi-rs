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
    static void throwIt(Uint8List buf) {
        print("1234567890");
        var bytes = ByteData.view(buf.buffer);
        int errorCode = bytes.getInt32(0);
        print("error code - $errorCode");

        switch (errorCode) {
            {% for variant in e.variants() %}
            case {{ type_name }}.{{ variant.name()|class_name }}:
                throw {{ type_name }}({{ type_name }}.{{ variant.name()|class_name }});
            {% endfor %}
            default:
                throw UniffiInternalError.unexpectedEnumCase;
        }
    }

    static T rustCallWithError<T>(Api api, Function(Pointer<RustCallStatus>) callback) {
        var callStatus = RustCallStatus.allocate();
        final returnValue = callback(callStatus);

        switch (callStatus.ref.code) {
            case CALL_SUCCESS:
                calloc.free(callStatus);
                return returnValue;
            case CALL_ERROR:
                throwIt(callStatus.ref.errorBuf.asByteBuffer());
                // fallback for the linter
                throw UniffiInternalError(callStatus.ref.code, null);
            case CALL_PANIC:
                if (callStatus.ref.errorBuf.len > 0) {
                    final message = {{ Type::String.borrow()|lift_fn }}(api, callStatus.ref.errorBuf);
                    calloc.free(callStatus);
                    throw UniffiInternalError.panicked(message);
                } else {
                    calloc.free(callStatus);
                    throw UniffiInternalError.panicked("Rust panic");
                }
            default:
                throw UniffiInternalError(callStatus.ref.code, null);
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
