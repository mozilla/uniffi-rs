{%- if func.is_async() %}

fileprivate class _UniFFI_{{ func.name()|class_name }}_Env {
    var rustFuture: OpaquePointer
    var continuation: CheckedContinuation<{% match func.return_type() %}{% when Some with (return_type) %}{{ return_type|type_name }}{% when None %}(){% endmatch %}, Never>

    init(rustyFuture: OpaquePointer, continuation: CheckedContinuation<{% match func.return_type() %}{% when Some with (return_type) %}{{ return_type|type_name }}{% when None %}(){% endmatch %}, Never>) {
        self.rustFuture = rustyFuture
        self.continuation = continuation
    }

    deinit {
        try! rustCall {
            {{ func.ffi_func().name() }}_drop(self.rustFuture, $0)
        }
    }
}

fileprivate func _UniFFI_{{ func.name()|class_name }}_waker(raw_env: UnsafeMutableRawPointer?) {
    Task {
        let env = Unmanaged<_UniFFI_{{ func.name()|class_name }}_Env>.fromOpaque(raw_env!)
        let env_ref = env.takeUnretainedValue()
        let polledResult = {% match func.ffi_func().return_type() -%}
        {%- when Some with (return_type) -%}
            UnsafeMutablePointer<{{ return_type|type_ffi_lowered }}>.allocate(capacity: 1)
        {%- when None -%}
            UnsafeMutableRawPointer.allocate(byteCount: 0, alignment: 0)
        {%- endmatch %}
        let isReady = try! rustCall() {
            {{ func.ffi_func().name() }}_poll(
                env_ref.rustFuture,
                _UniFFI_{{ func.name()|class_name }}_waker,
                env.toOpaque(),
                polledResult,
                $0
            )
        }

        if isReady {
            env_ref.continuation.resume(returning: {% match func.return_type() -%}
            {%- when Some with (return_type) -%}
                try! {{ return_type|lift_fn }}(polledResult.move())
            {%- when None -%}
                ()
            {%- endmatch -%}
            )
            polledResult.deallocate()
            env.release()
        }
    }
}

public func {{ func.name()|fn_name }}({%- call swift::arg_list_decl(func) -%}) async {% call swift::throws(func) %}{% match func.return_type() %}{% when Some with (return_type) %} -> {{ return_type|type_name }}{% when None %}{% endmatch %} {
    let future = {% call swift::to_ffi_call(func) %}

    return await withCheckedContinuation { continuation in
        let env = Unmanaged.passRetained(_UniFFI_{{ func.name()|class_name }}_Env(rustyFuture: future, continuation: continuation))
        _UniFFI_{{ func.name()|class_name }}_waker(raw_env: env.toOpaque())
    }
}

{%- else %}

{%- match func.return_type() -%}
{%- when Some with (return_type) %}

public func {{ func.name()|fn_name }}({%- call swift::arg_list_decl(func) -%}) {% call swift::throws(func) %} -> {{ return_type|type_name }} {
    return {% call swift::try(func) %} {{ return_type|lift_fn }}(
        {% call swift::to_ffi_call(func) %}
    )
}

{% when None %}

public func {{ func.name()|fn_name }}({% call swift::arg_list_decl(func) %}) {% call swift::throws(func) %} {
    {% call swift::to_ffi_call(func) %}
}
{% endmatch %}
{%- endif %}