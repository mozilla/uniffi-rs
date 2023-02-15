{%- if func.is_async() %}
{%- match func.throws_type() -%}
{%- when Some with (throwable) %}
@Throws({{ throwable|error_type_name }}::class)
{%- else -%}
{%- endmatch %}

@Suppress("ASSIGNED_BUT_NEVER_ACCESSED_VARIABLE")
suspend fun {{ func.name()|fn_name }}({%- call kt::arg_list_decl(func) -%}){% match func.return_type() %}{% when Some with (return_type) %} : {{ return_type|type_name }}{% when None %}{%- endmatch %} {
    // Create a new `CoroutineScope` for this operation, suspend the coroutine, and call the
    // scaffolding function, passing it one of the callback handlers from `AsyncTypes.kt`.
    return coroutineScope {
        val scope = this
        return@coroutineScope suspendCancellableCoroutine { continuation ->
            try {
                val callback = {{ func.result_type().borrow()|future_callback_handler }}(continuation)
                uniffiActiveFutureCallbacks.add(callback)
                continuation.invokeOnCancellation { uniffiActiveFutureCallbacks.remove(callback) }
                rustCall { status ->
                    _UniFFILib.INSTANCE.{{ func.ffi_func().name() }}(
                        {% call kt::arg_list_lowered(func) %}
                        FfiConverterForeignExecutor.lower(scope),
                        callback,
                        USize(0),
                        status,
                    )
                }
            } catch (e: Exception) {
                continuation.resumeWithException(e)
            }
        }
    }
}

{%- else %}
{%- match func.throws_type() -%}
{%- when Some with (throwable) %}
@Throws({{ throwable|error_type_name }}::class)
{%- else -%}
{%- endmatch -%}

{%- match func.return_type() -%}
{%- when Some with (return_type) %}

{% include "FunctionDocsTemplate.kt" %}
fun {{ func.name()|fn_name }}({%- call kt::arg_list_decl(func) -%}): {{ return_type|type_name }} {
    return {{ return_type|lift_fn }}({% call kt::to_ffi_call(func) %})
}
{% when None %}

{% include "FunctionDocsTemplate.kt" %}
fun {{ func.name()|fn_name }}({% call kt::arg_list_decl(func) %}) =
    {% call kt::to_ffi_call(func) %}

{% endmatch %}
{%- endif %}
