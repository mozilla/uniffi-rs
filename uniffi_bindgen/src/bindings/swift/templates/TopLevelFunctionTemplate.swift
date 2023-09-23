{%- if func.is_async() %}

public func {{ func.name()|fn_name }}({%- call swift::arg_list_decl(func) -%}) async {% call swift::throws(func) %}{% match func.return_type() %}{% when Some with (return_type) %} -> {{ return_type|type_name }}{% when None %}{% endmatch %} {
    {%- if func|is_cancellable(config) %}
    return try await uniffiRustCallAsyncCancellable(
    {%- else %}
    return {% if func.throws() %}try {% endif %}await uniffiRustCallAsync(
    {%- endif %}
        rustFutureFunc: {
            {{ func.ffi_func().name() }}(
                {%- for arg in func.arguments() %}
                {{ arg|lower_fn }}({{ arg.name()|var_name }}){% if !loop.last %},{% endif %}
                {%- endfor %}
            )
        },
        pollFunc: {{ func.ffi_rust_future_poll(ci) }},
        {%- if func|is_cancellable(config) %}
        cancelFunc: {{ func.ffi_rust_future_cancel(ci) }},
        {%- endif %}
        completeFunc: { rustFuture in
            {%- match func.return_type() %}
            {%- when Some(return_type) %}
            let liftReturn = { try! {{ return_type|lift_fn }}($0) }
            {%- when None %}
            let liftReturn = { (_: ()) in () }
            {%- endmatch %}

            return liftReturn(
                {%- match func.throws_type() %}
                {%- when Some with (e) %}
                try uniffiRustCallWithError(
                    {{ e|ffi_converter_name }}.lift
                ) { callStatus in
                    {{ func.ffi_rust_future_complete(ci) }}(rustFuture, callStatus)
                }
                {%- else %}
                uniffiRustCall { callStatus in
                    {{ func.ffi_rust_future_complete(ci) }}(rustFuture, callStatus)
                }
                {%- endmatch %}
            )

        },
        freeFunc: {{ func.ffi_rust_future_free(ci) }}
    )
}

{% else %}

{%- match func.return_type() -%}
{%- when Some with (return_type) %}

public func {{ func.name()|fn_name }}({%- call swift::arg_list_decl(func) -%}) {% call swift::throws(func) %} -> {{ return_type|type_name }} {
    return {% call swift::try(func) %} {{ return_type|lift_fn }}(
        {% call swift::to_ffi_call(func) %}
    )
}

{%- when None %}

public func {{ func.name()|fn_name }}({% call swift::arg_list_decl(func) %}) {% call swift::throws(func) %} {
    {% call swift::to_ffi_call(func) %}
}

{% endmatch %}
{%- endif %}
