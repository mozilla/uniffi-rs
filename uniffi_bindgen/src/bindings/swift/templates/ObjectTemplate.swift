{%- let obj = ci.get_object_definition(name).unwrap() %}
public protocol {{ obj.name() }}Protocol {
    {% for meth in obj.methods() -%}
    func {{ meth.name()|fn_name }}({% call swift::arg_list_protocol(meth) %}) {% call swift::async(meth) %} {% call swift::throws(meth) -%}
    {%- match meth.return_type() -%}
    {%- when Some with (return_type) %} -> {{ return_type|type_name -}}
    {%- else -%}
    {%- endmatch %}
    {% endfor %}
}

public class {{ type_name }}: {{ obj.name() }}Protocol {
    fileprivate let pointer: UnsafeMutableRawPointer

    // TODO: We'd like this to be `private` but for Swifty reasons,
    // we can't implement `FfiConverter` without making this `required` and we can't
    // make it `required` without making it `public`.
    required init(unsafeFromRawPointer pointer: UnsafeMutableRawPointer) {
        self.pointer = pointer
    }

    {%- match obj.primary_constructor() %}
    {%- when Some with (cons) %}
    public convenience init({% call swift::arg_list_decl(cons) -%}) {% call swift::throws(cons) %} {
        self.init(unsafeFromRawPointer: {% call swift::to_ffi_call(cons) %})
    }
    {%- when None %}
    {%- endmatch %}

    deinit {
        try! rustCall { {{ obj.ffi_object_free().name() }}(pointer, $0) }
    }

    {% for cons in obj.alternate_constructors() %}
    public static func {{ cons.name()|fn_name }}({% call swift::arg_list_decl(cons) %}) {% call swift::throws(cons) %} -> {{ type_name }} {
        return {{ type_name }}(unsafeFromRawPointer: {% call swift::to_ffi_call(cons) %})
    }
    {% endfor %}

    {# // TODO: Maybe merge the two templates (i.e the one with a return type and the one without) #}
    {% for meth in obj.methods() -%}
    {%- if meth.is_async() -%}

    public func {{ meth.name()|fn_name }}({%- call swift::arg_list_decl(meth) -%}) async {% call swift::throws(meth) %}{% match meth.return_type() %}{% when Some with (return_type) %} -> {{ return_type|type_name }}{% when None %}{% endmatch %} {
        let future = {% call swift::to_ffi_call_with_prefix("self.pointer", meth) %}

        return {% if meth.throws() -%}
            try await withCheckedThrowingContinuation
        {%- else -%}
            await withCheckedContinuation
        {%- endif -%}
        { continuation in
            let env = Unmanaged.passRetained(_UniFFI_{{ obj.name() }}_{{ meth.name()|class_name }}_Env(rustyFuture: future, continuation: continuation))
            _UniFFI_{{ obj.name() }}_{{ meth.name()|class_name }}_waker(raw_env: env.toOpaque())
        }
    }

    {% else -%}

    {%- match meth.return_type() -%}

    {%- when Some with (return_type) -%}
    public func {{ meth.name()|fn_name }}({% call swift::arg_list_decl(meth) %}) {% call swift::throws(meth) %} -> {{ return_type|type_name }} {
        return {% call swift::try(meth) %} {{ return_type|lift_fn }}(
            {% call swift::to_ffi_call_with_prefix("self.pointer", meth) %}
        )
    }

    {%- when None -%}
    public func {{ meth.name()|fn_name }}({% call swift::arg_list_decl(meth) %}) {% call swift::throws(meth) %} {
        {% call swift::to_ffi_call_with_prefix("self.pointer", meth) %}
    }
    {%- endmatch -%}
    {%- endif -%}
    {% endfor %}
}

{% for meth in obj.methods() -%}
{%- if meth.is_async() -%}

fileprivate class _UniFFI_{{ obj.name() }}_{{ meth.name()|class_name }}_Env {
    var rustFuture: OpaquePointer
    var continuation: CheckedContinuation<{% match meth.return_type() %}{% when Some with (return_type) %}{{ return_type|type_name }}{% when None %}(){% endmatch %}, {% if meth.throws() %}Error{% else %}Never{% endif %}>

    init(rustyFuture: OpaquePointer, continuation: CheckedContinuation<{% match meth.return_type() %}{% when Some with (return_type) %}{{ return_type|type_name }}{% when None %}(){% endmatch %}, {% if meth.throws() %}Error{% else %}Never{% endif %}>) {
        self.rustFuture = rustyFuture
        self.continuation = continuation
    }

    deinit {
        try! rustCall {
            {{ meth.ffi_func().name() }}_drop(self.rustFuture, $0)
        }
    }
}

fileprivate func _UniFFI_{{ obj.name() }}_{{ meth.name()|class_name }}_waker(raw_env: UnsafeMutableRawPointer?) {
    Task {
        let env = Unmanaged<_UniFFI_{{ obj.name() }}_{{ meth.name()|class_name }}_Env>.fromOpaque(raw_env!)
        let env_ref = env.takeUnretainedValue()
        let polledResult = {% match meth.ffi_func().return_type() -%}
        {%- when Some with (return_type) -%}
            UnsafeMutablePointer<{{ return_type|type_ffi_lowered }}>.allocate(capacity: 1)
        {%- when None -%}
            UnsafeMutableRawPointer.allocate(byteCount: 0, alignment: 0)
        {%- endmatch %}
        {% if meth.throws() -%}do {
        {%- endif %}

        let isReady = {% match meth.throws_type() -%}
        {%- when Some with (error) -%}
            try rustCallWithError({{ error|ffi_converter_name }}.self) {
        {%- when None -%}
            try! rustCall() {
        {%- endmatch %}
            {{ meth.ffi_func().name() }}_poll(
                env_ref.rustFuture,
                _UniFFI_{{ obj.name() }}_{{ meth.name()|class_name }}_waker,
                env.toOpaque(),
                polledResult,
                $0
            )
        }

        if isReady {
            env_ref.continuation.resume(returning: {% match meth.return_type() -%}
            {%- when Some with (return_type) -%}
                try! {{ return_type|lift_fn }}(polledResult.move())
            {%- when None -%}
                ()
            {%- endmatch -%}
            )
            polledResult.deallocate()
            env.release()
        }
        {%- if meth.throws() %}
        } catch {
            env_ref.continuation.resume(throwing: error)
            polledResult.deallocate()
            env.release()
        }
        {%- endif %}
    }
}

{% endif -%}
{%- endfor %}

public struct {{ ffi_converter_name }}: FfiConverter {
    typealias FfiType = UnsafeMutableRawPointer
    typealias SwiftType = {{ type_name }}

    public static func read(from buf: inout (data: Data, offset: Data.Index)) throws -> {{ type_name }} {
        let v: UInt64 = try readInt(&buf)
        // The Rust code won't compile if a pointer won't fit in a UInt64.
        // We have to go via `UInt` because that's the thing that's the size of a pointer.
        let ptr = UnsafeMutableRawPointer(bitPattern: UInt(truncatingIfNeeded: v))
        if (ptr == nil) {
            throw UniffiInternalError.unexpectedNullPointer
        }
        return try lift(ptr!)
    }

    public static func write(_ value: {{ type_name }}, into buf: inout [UInt8]) {
        // This fiddling is because `Int` is the thing that's the same size as a pointer.
        // The Rust code won't compile if a pointer won't fit in a `UInt64`.
        writeInt(&buf, UInt64(bitPattern: Int64(Int(bitPattern: lower(value)))))
    }

    public static func lift(_ pointer: UnsafeMutableRawPointer) throws -> {{ type_name }} {
        return {{ type_name}}(unsafeFromRawPointer: pointer)
    }

    public static func lower(_ value: {{ type_name }}) -> UnsafeMutableRawPointer {
        return value.pointer
    }
}

{#
We always write these public functions just in case the enum is used as
an external type by another crate.
#}
public func {{ ffi_converter_name }}_lift(_ pointer: UnsafeMutableRawPointer) throws -> {{ type_name }} {
    return try {{ ffi_converter_name }}.lift(pointer)
}

public func {{ ffi_converter_name }}_lower(_ value: {{ type_name }}) -> UnsafeMutableRawPointer {
    return {{ ffi_converter_name }}.lower(value)
}
