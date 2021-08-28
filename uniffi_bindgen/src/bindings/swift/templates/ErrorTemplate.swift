// An error type for FFI errors. These errors occur at the UniFFI level, not
// the library level.
fileprivate enum UniffiInternalError: LocalizedError {
    case bufferOverflow
    case incompleteData
    case unexpectedOptionalTag
    case unexpectedEnumCase
    case unexpectedNullPointer
    case unexpectedRustCallStatusCode
    case unexpectedRustCallError
    case rustPanic(_ message: String)

    public var errorDescription: String? {
        switch self {
        case .bufferOverflow: return "Reading the requested value would read past the end of the buffer"
        case .incompleteData: return "The buffer still has data after lifting its containing value"
        case .unexpectedOptionalTag: return "Unexpected optional tag; should be 0 or 1"
        case .unexpectedEnumCase: return "Raw enum value doesn't match any cases"
        case .unexpectedNullPointer: return "Raw pointer value was null"
        case .unexpectedRustCallStatusCode: return "Unexpected RustCallStatus code"
        case .unexpectedRustCallError: return "CALL_ERROR but no errorClass specified"
        case let .rustPanic(message): return message
        }
    }
}

fileprivate let CALL_SUCCESS: Int8 = 0
fileprivate let CALL_ERROR: Int8 = 1
fileprivate let CALL_PANIC: Int8 = 2

fileprivate extension RustCallStatus {
    init() {
        self.init(
            code: CALL_SUCCESS,
            errorBuf: RustBuffer.init(
                capacity: 0,
                len: 0,
                data: nil
            )
        )
    }
}

{# Define enums to handle each individual error #}
{% for e in ci.iter_error_definitions() %}
public enum {{ e.name()|class_name_swift }} {

    {% if e.is_flat() %}
    {% for variant in e.variants() %}
    // Simple error enums only carry a message
    case {{ variant.name()|class_name_swift }}(message: String)
    {% endfor %}

    {%- else %}
    {% for variant in e.variants() %}
    case {{ variant.name()|class_name_swift }}{% if variant.fields().len() > 0 %}({% call swift::field_list_decl(variant) %}){% endif -%}
    {% endfor %}

    {%- endif %}
}

extension {{ e.name()|class_name_swift }}: ViaFfiUsingByteBuffer, ViaFfi {
    fileprivate static func read(from buf: Reader) throws -> {{ e.name()|class_name_swift }} {
        let variant: Int32 = try buf.readInt()
        switch variant {

        {% if e.is_flat() %}

        {% for variant in e.variants() %}
        case {{ loop.index }}: return .{{ variant.name()|class_name_swift }}(
            message: try String.read(from: buf)
        )
        {% endfor %}

       {% else %}

        {% for variant in e.variants() %}
        case {{ loop.index }}: return .{{ variant.name()|class_name_swift }}{% if variant.has_fields() -%}(
            {% for field in variant.fields() -%}
            {{ field.name()|var_name_swift }}: try {{ "buf"|read_swift(field.type_()) }}{% if loop.last %}{% else %},{% endif %}
            {% endfor -%}
        ){% endif -%}
        {% endfor %}

         {% endif -%}
        default: throw UniffiInternalError.unexpectedEnumCase
        }
    }

    fileprivate func write(into buf: Writer) {
        switch self {

        {% if e.is_flat() %}

        {% for variant in e.variants() %}
        case let .{{ variant.name()|class_name_swift }}(message):
            buf.writeInt(Int32({{ loop.index }}))
            message.write(into: buf)
        {%- endfor %}

        {% else %}

        {% for variant in e.variants() %}
        {% if variant.has_fields() %}
        case let .{{ variant.name()|class_name_swift }}({% for field in variant.fields() %}{{ field.name()|var_name_swift }}{%- if loop.last -%}{%- else -%},{%- endif -%}{% endfor %}):
            buf.writeInt(Int32({{ loop.index }}))
            {% for field in variant.fields() -%}
            {{ field.name()|var_name_swift }}.write(into: buf)
            {% endfor -%}
        {% else %}
        case .{{ variant.name()|class_name_swift }}:
            buf.writeInt(Int32({{ loop.index }}))
        {% endif %}
        {%- endfor %}

        {%- endif %}
        }
    }
}

{% if !ci.item_contains_object_references(e) %}
extension {{ e.name()|class_name_swift }}: Equatable, Hashable {}
{% endif %}
extension {{ e.name()|class_name_swift }}: Error { }
{% endfor %}

private func rustCall<T>(_ callback: (UnsafeMutablePointer<RustCallStatus>) -> T) throws -> T {
    try makeRustCall(callback, errorHandler: {
        $0.deallocate()
        return UniffiInternalError.unexpectedRustCallError
    })
}

private func rustCallWithError<T, E: ViaFfiUsingByteBuffer & Error>(_ errorClass: E.Type, _ callback: (UnsafeMutablePointer<RustCallStatus>) -> T) throws -> T {
    try makeRustCall(callback, errorHandler: { return try E.lift($0) })
}

private func makeRustCall<T>(_ callback: (UnsafeMutablePointer<RustCallStatus>) -> T, errorHandler: (RustBuffer) throws -> Error) throws -> T {
    var callStatus = RustCallStatus.init()
    let returnedVal = callback(&callStatus)
    switch callStatus.code {
        case CALL_SUCCESS:
            return returnedVal

        case CALL_ERROR:
            throw try errorHandler(callStatus.errorBuf)

        case CALL_PANIC:
            // When the rust code sees a panic, it tries to construct a RustBuffer
            // with the message.  But if that code panics, then it just sends back
            // an empty buffer.
            if callStatus.errorBuf.len > 0 {
                throw UniffiInternalError.rustPanic(try String.lift(callStatus.errorBuf))
            } else {
                callStatus.errorBuf.deallocate()
                throw UniffiInternalError.rustPanic("Rust panic")
            }

        default:
            throw UniffiInternalError.unexpectedRustCallStatusCode
    }
}
