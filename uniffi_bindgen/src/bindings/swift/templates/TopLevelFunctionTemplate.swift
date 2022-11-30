{%- if func.is_async() %}
{%- match func.return_type() -%}
{%- when Some with (return_type) %}
public func {{ func.name()|fn_name }}({%- call swift::arg_list_decl(func) -%}) async {% call swift::throws(func) %} -> {{ return_type|type_name }} {
    let rustFuture = {% call swift::to_ffi_call(func) %}

    struct Env {
        var rustFuture: UnsafePointer<RustFuture>
        var continuation: CheckedContinuation<Bool, Never>
    }
    
    return await withCheckedContinuation { continuation in
        func waker(env: UnsafePointer<Env>) {
            let polledResult = UnsafeMutablePointer<Bool>.allocate(capacity: 1)
            let isReady = try! rustCall() {
                {{ func.ffi_func().name() }}_poll(env.pointee.rustFuture, waker, env, polledResult, $0)
            }

            if isReady {
                try! rustCall {
                    {{ func.ffi_func().name() }}_drop(rustFuture, $0)
                }
                env.pointee.continuation.resume(with: {{ return_type|lift_fn }}(polledResult))
            }
        }
        
        let env = Env {
            rustFuture = rustFuture
            continuation = continuation
        }

        waker(env: &env)
    }
}

{%- when None %}

// TODO: {{ func.name()|fn_name }} is async but doesn't return anything

{% endmatch %}
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