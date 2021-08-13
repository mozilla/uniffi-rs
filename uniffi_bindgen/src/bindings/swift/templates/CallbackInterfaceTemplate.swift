{% import "macros.swift" as swift %}
{% let cbi = self.inner() %}
{% let type_name = cbi.name()|class_name_swift %}
public protocol {{ type_name }} : AnyObject {
    {% for meth in cbi.methods() -%}
    func {{ meth.name()|fn_name_swift }}({% call swift::arg_list_protocol(meth) %}) {% call swift::throws(meth) -%}
    {%- match meth.return_type() -%}
    {%- when Some with (return_type) %} -> {{ return_type|type_swift -}}
    {%- else -%}
    {%- endmatch %}
    {% endfor %}
}

{% let canonical_type_name = cbi.type_().canonical_name()|class_name_swift %}
{% let callback_internals = format!("{}Internals", canonical_type_name) -%}
{% let callback_interface_impl = format!("{}FFI", canonical_type_name) -%}

let {{ callback_interface_impl }} : ForeignCallback =
    { (handle: UInt64, method: Int32, args: RustBuffer) -> RustBuffer in

        {% for meth in cbi.methods() -%}
    {% let method_name = format!("invoke_{}", meth.name())|fn_name_swift %}

    func {{ method_name }}(_ swiftCallbackInterface: {{ type_name }}, _ args: RustBuffer) throws -> RustBuffer {
        defer { args.deallocate() }
        {#- Unpacking args from the RustBuffer #}
            {%- if meth.arguments().len() != 0 -%}
            {#- Calling the concrete callback object #}

            let reader = Reader(data: Data(rustBuffer: args))
            let result = swiftCallbackInterface.{{ meth.name()|fn_name_swift }}(
                    {% for arg in meth.arguments() -%}
                    {{ arg.name() }}: try {{ "reader"|read_swift(arg.type_()) }}
                    {%- if !loop.last %}, {% endif %}
                    {% endfor -%}
                )
            {% else %}
            let result = swiftCallbackInterface.{{ meth.name()|fn_name_swift }}()
            {% endif -%}

        {#- Packing up the return value into a RustBuffer #}
                {%- match meth.return_type() -%}
                {%- when Some with (return_type) -%}
                let writer = Writer()
                result.write(into: writer)
                return RustBuffer(bytes: writer.bytes)
                {%- else -%}
                return RustBuffer()
                {% endmatch -%}
                // TODO catch errors and report them back to Rust.
                // https://github.com/mozilla/uniffi-rs/issues/351

    }
    {% endfor %}

        return {{ callback_internals }}.handleMap.callWithResult(handle: handle) { cb -> RustBuffer in
            switch method {
                case IDX_CALLBACK_FREE: return {{ callback_internals }}.drop(handle: handle)
                {% for meth in cbi.methods() -%}
                {% let method_name = format!("invoke_{}", meth.name())|fn_name_swift -%}
                case {{ loop.index }}: return try! {{ method_name }}(cb, args)
                {% endfor %}
                // This should never happen, because an out of bounds method index won't
                // ever be used. Once we can catch errors, we should return an InternalError.
                // https://github.com/mozilla/uniffi-rs/issues/351
                default: return RustBuffer()
            }
        }
    }

let {{ callback_internals }} = _{{ callback_internals }}()
class _{{ callback_internals }}<T: {{ type_name }}>: CallbackInternals<T> {

    init() {
        super.init(foreignCallback: {{ callback_interface_impl }})
        register()
    }
    func register() {
        try? rustCall { (err: UnsafeMutablePointer<RustCallStatus>) in
            {{ cbi.ffi_init_callback().name() }}(self.foreignCallback, err)
        }
    }
}
