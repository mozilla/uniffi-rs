{%- let cbi = ci.get_callback_interface_definition(name).unwrap() %}
{%- let foreign_callback = format!("foreignCallback{}", canonical_type_name) %}
{%- if self.include_once_check("CallbackInterfaceRuntime.swift") %}{%- include "CallbackInterfaceRuntime.swift" %}{%- endif %}

// Declaration and FfiConverters for {{ type_name }} Callback Interface

public protocol {{ type_name }} : AnyObject {
    {% for meth in cbi.methods() -%}
    func {{ meth.name()|fn_name }}({% call swift::arg_list_protocol(meth) %}) {% call swift::throws(meth) -%}
    {%- match meth.return_type() -%}
    {%- when Some with (return_type) %} -> {{ return_type|type_name -}}
    {%- else -%}
    {%- endmatch %}
    {% endfor %}
}

// The ForeignCallback that is passed to Rust.
{%- if new_callback_interface_abi %}
fileprivate let {{ foreign_callback }} : ForeignCallback =
    { (handle: UniFFICallbackHandle, method: Int32, argsData: UnsafePointer<UInt8>, argsLen: Int32, out_buf: UnsafeMutablePointer<RustBuffer>) -> Int32 in
{%- else %}
fileprivate let {{ foreign_callback }} : ForeignCallback =
    { (handle: UniFFICallbackHandle, method: Int32, args: RustBuffer, out_buf: UnsafeMutablePointer<RustBuffer>) -> Int32 in
{%- endif %}
    {% for meth in cbi.methods() -%}
    {%- let method_name = format!("invoke_{}", meth.name())|fn_name -%}

    {%- if new_callback_interface_abi %}
    func {{ method_name }}(_ swiftCallbackInterface: {{ type_name }}, _ argsData: UnsafePointer<UInt8>, _ argsLen: Int32, _ out_buf: UnsafeMutablePointer<RustBuffer>) throws -> Int32 {
    {%- else %}
    func {{ method_name }}(_ swiftCallbackInterface: {{ type_name }}, _ args: RustBuffer, _ out_buf: UnsafeMutablePointer<RustBuffer>) throws -> Int32 {
    {%- endif %}
        {%- if !new_callback_interface_abi %}
        defer { args.deallocate() }
        {%- endif %}

        {%- if meth.arguments().len() > 0 %}
        {%- if new_callback_interface_abi %}
        var reader = createReader(data: Data(bytes: argsData, count: Int(argsLen)))
        {%- else %}
        var reader = createReader(data: Data(rustBuffer: args))
        {%- endif %}
        {%- endif %}

        {%- match meth.return_type() %}
        {%- when Some(return_type) %}
        func makeCall() throws -> Int32 {
            let result = try swiftCallbackInterface.{{ meth.name()|fn_name }}(
                    {% for arg in meth.arguments() -%}
                    {% if !config.omit_argument_labels() %}{{ arg.name()|var_name }}: {% endif %} try {{ arg|read_fn }}(from: &reader)
                    {%- if !loop.last %}, {% endif %}
                    {% endfor -%}
                )
            var writer = [UInt8]()
            {{ return_type|write_fn }}(result, into: &writer)
            out_buf.pointee = RustBuffer(bytes: writer)
            return 1
        }
        {%- when None %}
        func makeCall() throws -> Int32 {
            try swiftCallbackInterface.{{ meth.name()|fn_name }}(
                    {% for arg in meth.arguments() -%}
                    {% if !config.omit_argument_labels() %}{{ arg.name()|var_name }}: {% endif %} try {{ arg|read_fn }}(from: &reader)
                    {%- if !loop.last %}, {% endif %}
                    {% endfor -%}
                )
            return 1
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
            return -2
        }
        {%- endmatch %}
    }
    {%- endfor %}


    switch method {
        case IDX_CALLBACK_FREE:
            {{ ffi_converter_name }}.drop(handle: handle)
            // No return value.
            // See docs of ForeignCallback in `uniffi/src/ffi/foreigncallbacks.rs`
            return 0
        {% for meth in cbi.methods() -%}
        {% let method_name = format!("invoke_{}", meth.name())|fn_name -%}
        case {{ loop.index }}:
            let cb: {{ cbi|type_name }}
            do {
                cb = try {{ ffi_converter_name }}.lift(handle)
            } catch {
                out_buf.pointee = {{ Type::String.borrow()|lower_fn }}("{{ cbi.name() }}: Invalid handle")
                return -1
            }
            do {
                {%- if new_callback_interface_abi %}
                return try {{ method_name }}(cb, argsData, argsLen, out_buf)
                {%- else %}
                return try {{ method_name }}(cb, args, out_buf)
                {%- endif %}
            } catch let error {
                out_buf.pointee = {{ Type::String.borrow()|lower_fn }}(String(describing: error))
                return -1
            }
        {% endfor %}
        // This should never happen, because an out of bounds method index won't
        // ever be used. Once we can catch errors, we should return an InternalError.
        // https://github.com/mozilla/uniffi-rs/issues/351
        default:
            // An unexpected error happened.
            // See docs of ForeignCallback in `uniffi/src/ffi/foreigncallbacks.rs`
            return -1
    }
}

// FfiConverter protocol for callback interfaces
fileprivate struct {{ ffi_converter_name }} {
    // Initialize our callback method with the scaffolding code
    private static var callbackInitialized = false
    private static func initCallback() {
        try! rustCall { (err: UnsafeMutablePointer<RustCallStatus>) in
                {%- if new_callback_interface_abi %}
                {{ cbi.ffi_init_callback2().name() }}({{ foreign_callback }}, err)
                {%- else %}
                {{ cbi.ffi_init_callback().name() }}({{ foreign_callback }}, err)
                {%- endif %}
        }
    }
    private static func ensureCallbackinitialized() {
        if !callbackInitialized {
            initCallback()
            callbackInitialized = true
        }
    }

    static func drop(handle: UniFFICallbackHandle) {
        handleMap.remove(handle: handle)
    }

    private static var handleMap = UniFFICallbackHandleMap<{{ type_name }}>()
}

extension {{ ffi_converter_name }} : FfiConverter {
    typealias SwiftType = {{ type_name }}
    // We can use Handle as the FfiType because it's a typealias to UInt64
    typealias FfiType = UniFFICallbackHandle

    public static func lift(_ handle: UniFFICallbackHandle) throws -> SwiftType {
        ensureCallbackinitialized();
        guard let callback = handleMap.get(handle: handle) else {
            throw UniffiInternalError.unexpectedStaleHandle
        }
        return callback
    }

    public static func read(from buf: inout (data: Data, offset: Data.Index)) throws -> SwiftType {
        ensureCallbackinitialized();
        let handle: UniFFICallbackHandle = try readInt(&buf)
        return try lift(handle)
    }

    public static func lower(_ v: SwiftType) -> UniFFICallbackHandle {
        ensureCallbackinitialized();
        return handleMap.insert(obj: v)
    }

    public static func write(_ v: SwiftType, into buf: inout [UInt8]) {
        ensureCallbackinitialized();
        writeInt(&buf, lower(v))
    }
}
