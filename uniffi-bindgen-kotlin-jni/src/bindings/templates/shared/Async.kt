const val UNIFFI_RUST_FUTURE_PENDING = 0
const val UNIFFI_RUST_FUTURE_CANCELLED = 1
const val UNIFFI_RUST_FUTURE_COMPLETE = 2

fun uniffiContinuationResume(continuation: kotlin.coroutines.Continuation<kotlin.Boolean>) {
    continuation.resumeWith(Result.success(false))
}

{%- for rust_result in root.rust_async_callable_results() %}
suspend fun {{ rust_result.async_await_future_fn() }}(
    rustFuture: kotlin.Long,
){%- if let Some(return_type) = rust_result.return_type %} : {{ return_type.type_kt }}{% endif %}
{
    try {
        {%- if rust_result.return_type.is_some() %}
        val completion = {{ rust_result.async_complete_class() }}();
        {%- endif %}
        while(true) {
            val futureReady = kotlin.coroutines.suspendCoroutine<kotlin.Boolean> { continuation ->
                val pollResult = Scaffolding.{{ rust_result.async_poll_fn() }}(
                    rustFuture,
                    continuation,
                    {%- if rust_result.return_type.is_some() %}
                    completion,
                    {%- endif %}
            )
                when (pollResult) {
                    UNIFFI_RUST_FUTURE_PENDING -> {
                        // Don't do anything at this point.
                        // The contiunation will be resumed when the Rust waker is called
                        // and Rust calls the `uniffiContinuationResume`
                    }
                    UNIFFI_RUST_FUTURE_COMPLETE -> continuation.resumeWith(Result.success(true))
                    UNIFFI_RUST_FUTURE_CANCELLED -> continuation.resumeWith(
                        Result.failure(kotlin.coroutines.cancellation.CancellationException())
                    )
                    else -> continuation.resumeWith(
                        Result.failure(uniffi.InternalException("Error polling Rust future (code: $pollResult)"))
                    )
                }
            }
            if (futureReady) {
                {%- if rust_result.return_type.is_some() %}
                return completion.value!!
                {%- else %}
                return
                {%- endif %}
            }
        }
    } finally {
        Scaffolding.{{ rust_result.async_free_fn() }}(rustFuture)
    }
}

{%- if let Some(return_type) = rust_result.return_type %}
class {{ rust_result.async_complete_class() }} {
    var value: {{ return_type.type_kt }}? = null;
    fun complete(
        {%- for ffi_type in return_type.ffi_types %}
        v{{loop.index0 }}: {{ ffi_type.type_kt() }},
        {%- endfor %}
    ) {
        this.value = {{ return_type.lift_fn_kt() }}(
            {%- for _ in return_type.ffi_types %}
            v{{loop.index0 }},
            {%- endfor %}
        )
    }
}
{%- endif %}
{%- endfor %}
