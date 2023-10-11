{%- if self.include_once_check("CallbackInterfaceRuntime.swift") %}{%- include "CallbackInterfaceRuntime.swift" %}{%- endif %}

// Declaration and FfiConverters for {{ type_name }} Callback Interface

fileprivate let {{ callback_handler }} : ForeignCallback =
    { (handle: UniFFICallbackHandle, method: Int32, argsData: UnsafePointer<UInt8>, argsLen: Int32, out_buf: UnsafeMutablePointer<RustBuffer>) -> Int32 in
    {% for meth in methods.iter() -%}
    {%- let method_name = format!("invoke_{}", meth.name())|fn_name %}

    func {{ method_name }}(_ swiftCallbackInterface: {{ type_name }}, _ argsData: UnsafePointer<UInt8>, _ argsLen: Int32, _ out_buf: UnsafeMutablePointer<RustBuffer>) throws -> Int32 {
        {%- if meth.arguments().len() > 0 %}
        var reader = createReader(data: Data(bytes: argsData, count: Int(argsLen)))
        {%- endif %}

        {%- match meth.return_type() %}
        {%- when Some(return_type) %}
        func makeCall() throws -> Int32 {
            let result = {% if meth.throws() %} try{% endif %} swiftCallbackInterface.{{ meth.name()|fn_name }}(
                    {% for arg in meth.arguments() -%}
                    {% if !config.omit_argument_labels() %}{{ arg.name()|var_name }}: {% endif %} try {{ arg|read_fn }}(from: &reader)
                    {%- if !loop.last %}, {% endif %}
                    {% endfor -%}
                )
            var writer = [UInt8]()
            {{ return_type|write_fn }}(result, into: &writer)
            out_buf.pointee = RustBuffer(bytes: writer)
            return UNIFFI_CALLBACK_SUCCESS
        }
        {%- when None %}
        func makeCall() throws -> Int32 {
            {% if meth.throws() %}try {% endif %}swiftCallbackInterface.{{ meth.name()|fn_name }}(
                    {% for arg in meth.arguments() -%}
                    {% if !config.omit_argument_labels() %}{{ arg.name()|var_name }}: {% endif %} try {{ arg|read_fn }}(from: &reader)
                    {%- if !loop.last %}, {% endif %}
                    {% endfor -%}
                )
            return UNIFFI_CALLBACK_SUCCESS
        }
        {%- endmatch %}

        {%- match meth.throws_type() %}
        {%- when None %}
        return try makeCall()
        {%- when Some(error_type) %}
        do {
            return try makeCall()
        } catch let error as {{ error_type|type_name }} {
            out_buf.pointee = {{ error_type|lower_fn }}(error)
            return UNIFFI_CALLBACK_ERROR
        }
        {%- endmatch %}
    }
    {%- endfor %}


    switch method {
        case IDX_CALLBACK_FREE:
            {{ ffi_converter_name }}.handleMap.remove(handle: handle)
            // Sucessful return
            // See docs of ForeignCallback in `uniffi_core/src/ffi/foreigncallbacks.rs`
            return UNIFFI_CALLBACK_SUCCESS
        {% for meth in methods.iter() -%}
        {% let method_name = format!("invoke_{}", meth.name())|fn_name -%}
        case {{ loop.index }}:
            guard let cb = {{ ffi_converter_name }}.handleMap.get(handle: handle) else {
                out_buf.pointee = {{ Type::String.borrow()|lower_fn }}("No callback in handlemap; this is a Uniffi bug")
                return UNIFFI_CALLBACK_UNEXPECTED_ERROR
            }
            do {
                return try {{ method_name }}(cb, argsData, argsLen, out_buf)
            } catch let error {
                out_buf.pointee = {{ Type::String.borrow()|lower_fn }}(String(describing: error))
                return UNIFFI_CALLBACK_UNEXPECTED_ERROR
            }
        {% endfor %}
        // This should never happen, because an out of bounds method index won't
        // ever be used. Once we can catch errors, we should return an InternalError.
        // https://github.com/mozilla/uniffi-rs/issues/351
        default:
            // An unexpected error happened.
            // See docs of ForeignCallback in `uniffi_core/src/ffi/foreigncallbacks.rs`
            return UNIFFI_CALLBACK_UNEXPECTED_ERROR
    }
}

private func {{ callback_init }}() {
    {{ ffi_init_callback.name() }}({{ callback_handler }})
}
